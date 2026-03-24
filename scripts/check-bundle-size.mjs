import fs from "node:fs";
import path from "node:path";

const distAssetsDir = path.resolve(process.cwd(), "dist", "assets");

const BUDGETS = [
  { name: "entry", pattern: /^index-.*\.js$/, maxBytes: 230 * 1024 },
  {
    name: "vendor-react",
    pattern: /^vendor-react-.*\.js$/,
    maxBytes: 60 * 1024,
  },
  {
    name: "vendor-charts",
    pattern: /^vendor-charts-.*\.js$/,
    maxBytes: 390 * 1024,
  },
];

const MAX_ROUTE_CHUNK_BYTES = 20 * 1024;
const MAX_TOTAL_JS_BYTES = 730 * 1024;

function formatKiB(bytes) {
  return `${(bytes / 1024).toFixed(2)} KiB`;
}

function fail(message) {
  console.error(`\nBundle size check failed: ${message}`);
  process.exit(1);
}

if (!fs.existsSync(distAssetsDir)) {
  fail(`Missing build output at ${distAssetsDir}`);
}

const jsFiles = fs
  .readdirSync(distAssetsDir)
  .filter((file) => file.endsWith(".js"))
  .map((file) => {
    const fullPath = path.join(distAssetsDir, file);
    return { file, bytes: fs.statSync(fullPath).size };
  });

if (jsFiles.length === 0) {
  fail("No JavaScript files found in dist/assets");
}

for (const budget of BUDGETS) {
  const match = jsFiles.find(({ file }) => budget.pattern.test(file));
  if (!match) {
    fail(`Expected chunk for "${budget.name}" was not found`);
  }
  if (match.bytes > budget.maxBytes) {
    fail(
      `"${match.file}" is ${formatKiB(match.bytes)} (limit: ${formatKiB(
        budget.maxBytes,
      )})`,
    );
  }
}

const routeChunks = jsFiles.filter(
  ({ file }) => !/^index-.*\.js$/.test(file) && !/^vendor-.*\.js$/.test(file),
);

if (routeChunks.length > 0) {
  const largestRouteChunk = routeChunks.reduce((largest, current) =>
    current.bytes > largest.bytes ? current : largest,
  );
  if (largestRouteChunk.bytes > MAX_ROUTE_CHUNK_BYTES) {
    fail(
      `Largest route chunk "${largestRouteChunk.file}" is ${formatKiB(
        largestRouteChunk.bytes,
      )} (limit: ${formatKiB(MAX_ROUTE_CHUNK_BYTES)})`,
    );
  }
}

const totalJsBytes = jsFiles.reduce((sum, chunk) => sum + chunk.bytes, 0);
if (totalJsBytes > MAX_TOTAL_JS_BYTES) {
  fail(
    `Total JS size is ${formatKiB(totalJsBytes)} (limit: ${formatKiB(
      MAX_TOTAL_JS_BYTES,
    )})`,
  );
}

console.log("Bundle size check passed.");
