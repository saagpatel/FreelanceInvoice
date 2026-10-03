import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const ENTRY_LIMIT = 230 * 1024;
// Keep the original entry (230 KiB) + React (60 KiB) allowance, independent
// of where the bundler places the renderer, router, scheduler or shared code.
const BOOTSTRAP_LIMIT = (230 + 60) * 1024;
const CHARTS_LIMIT = 390 * 1024;
const ROUTE_LIMIT = 20 * 1024;
const TOTAL_LIMIT = 730 * 1024;
const isJs = (file) => /\.(?:js|mjs|cjs)$/.test(file);
const formatKiB = (bytes) => `${(bytes / 1024).toFixed(2)} KiB`;

function requireLimit(name, bytes, limit) {
  if (bytes > limit) {
    throw new Error(
      `${name} is ${formatKiB(bytes)} (limit: ${formatKiB(limit)})`,
    );
  }
}

function attributes(tag) {
  return Object.fromEntries(
    [...tag.matchAll(/([\w-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/g)].map(
      ([, name, double, single, bare]) => [
        name.toLowerCase(),
        double ?? single ?? bare,
      ],
    ),
  );
}

export function checkBundleSize(distDir = path.resolve("dist")) {
  const root = path.resolve(distDir);
  const files = new Map();
  function scan(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isSymbolicLink()) throw new Error(`Symlink in output: ${full}`);
      if (entry.isDirectory()) scan(full);
      else if (isJs(entry.name)) {
        const file = path.relative(root, full).split(path.sep).join("/");
        files.set(file, { file, bytes: fs.statSync(full).size });
      }
    }
  }
  scan(root);
  if (!files.size) throw new Error("No JavaScript files found in dist");

  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, ".vite", "manifest.json"), "utf8"),
  );
  if (!manifest || Array.isArray(manifest) || typeof manifest !== "object") {
    throw new Error("Invalid build manifest");
  }
  const byFile = new Map();
  for (const [key, chunk] of Object.entries(manifest)) {
    if (
      !chunk ||
      typeof chunk.file !== "string" ||
      !files.has(chunk.file) ||
      byFile.has(chunk.file)
    ) {
      throw new Error(`Missing or invalid manifest chunk: ${key}`);
    }
    byFile.set(chunk.file, key);
    for (const field of ["imports", "dynamicImports"]) {
      if (
        chunk[field] !== undefined &&
        (!Array.isArray(chunk[field]) ||
          chunk[field].some(
            (dependency) =>
              typeof dependency !== "string" ||
              !Object.hasOwn(manifest, dependency),
          ))
      ) {
        throw new Error(`Missing or invalid ${field} graph for: ${key}`);
      }
    }
  }
  for (const file of files.keys()) {
    if (!byFile.has(file))
      throw new Error(`Untracked JavaScript chunk: ${file}`);
  }
  const roots = new Set(
    Object.keys(manifest).filter((key) => manifest[key].isEntry === true),
  );
  if (!roots.size) throw new Error("No entry in build manifest");
  const html = fs
    .readFileSync(path.join(root, "index.html"), "utf8")
    .replace(/<!--[\s\S]*?-->/g, "");
  let moduleScripts = 0;
  for (const [tag] of html.matchAll(/<(?:script|link)\b[^>]*>/gi)) {
    const attrs = attributes(tag);
    const script = /^<script\b/i.test(tag) && attrs.type === "module";
    const preload =
      /^<link\b/i.test(tag) &&
      attrs.rel?.toLowerCase().split(/\s+/).includes("modulepreload");
    if (!script && !preload) continue;
    if (script) moduleScripts++;
    const source = script ? attrs.src : attrs.href;
    if (!source || /^(?:[a-z]+:|\/\/)/i.test(source)) {
      throw new Error("Expected a local external module script/preload");
    }
    const file = source.replace(/^\/?(?:\.\/)?/, "").split(/[?#]/)[0];
    if (!byFile.has(file))
      throw new Error(`HTML module missing from manifest: ${source}`);
    roots.add(byFile.get(file));
  }
  if (!moduleScripts) throw new Error("No module script in index.html");
  const visited = new Set();
  function visit(key) {
    if (visited.has(key)) return;
    visited.add(key);
    for (const dependency of manifest[key].imports ?? []) visit(dependency);
  }
  for (const key of roots) visit(key);
  const bootstrapFiles = [...visited].map((key) => manifest[key].file).sort();
  const bootstrapBytes = bootstrapFiles.reduce(
    (sum, file) => sum + files.get(file).bytes,
    0,
  );
  // All static imports and HTML preloads count, regardless of chunk names.
  requireLimit("Bootstrap JS", bootstrapBytes, BOOTSTRAP_LIMIT);

  const chunks = [...files.values()];
  function group(pattern, name, limit) {
    const matches = chunks.filter(({ file }) =>
      pattern.test(path.basename(file)),
    );
    if (!matches.length)
      throw new Error(`Expected chunk for "${name}" was not found`);
    const bytes = matches.reduce((sum, chunk) => sum + chunk.bytes, 0);
    if (limit !== undefined) requireLimit(name, bytes, limit);
    return bytes;
  }
  const entryBytes = group(/^index-.*\.js$/, "entry JS", ENTRY_LIMIT);
  group(/^vendor-react-.*\.js$/, "vendor-react");
  const chartsBytes = group(
    /^vendor-charts-.*\.js$/,
    "charts JS",
    CHARTS_LIMIT,
  );
  for (const chunk of chunks) {
    if (!/^(?:index-|vendor-)/.test(path.basename(chunk.file))) {
      requireLimit(
        `Route/shared chunk "${chunk.file}"`,
        chunk.bytes,
        ROUTE_LIMIT,
      );
    }
  }
  const totalBytes = chunks.reduce((sum, chunk) => sum + chunk.bytes, 0);
  requireLimit("Total JS", totalBytes, TOTAL_LIMIT);
  return {
    bootstrapBytes,
    bootstrapFiles,
    entryBytes,
    chartsBytes,
    totalBytes,
  };
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  try {
    const report = checkBundleSize();
    console.log(
      `Bundle size check passed: bootstrap ${formatKiB(report.bootstrapBytes)}, total ${formatKiB(report.totalBytes)} (raw JS).`,
    );
  } catch (error) {
    console.error(`\nBundle size check failed: ${error.message}`);
    process.exitCode = 1;
  }
}
