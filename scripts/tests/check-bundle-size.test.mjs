import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { checkBundleSize } from "../check-bundle-size.mjs";

const KiB = 1024;
function fixture(t, extra = {}, preloads = []) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "bundle-budget-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  fs.mkdirSync(path.join(dir, "assets"));
  fs.mkdirSync(path.join(dir, ".vite"));
  const chunks = {
    entry: {
      file: "index-entry.js",
      bytes: 40 * KiB,
      isEntry: true,
      imports: ["react"],
    },
    react: { file: "vendor-react-core.js", bytes: 200 * KiB },
    charts: { file: "vendor-charts-library.js", bytes: 360 * KiB },
    route: { file: "ClientsPage-route.js", bytes: 10 * KiB },
    ...extra,
  };
  const manifest = {};
  for (const [key, { bytes, file, ...graph }] of Object.entries(chunks)) {
    fs.mkdirSync(path.dirname(path.join(dir, "assets", file)), {
      recursive: true,
    });
    fs.writeFileSync(path.join(dir, "assets", file), "x".repeat(bytes));
    manifest[key] = { file: `assets/${file}`, ...graph };
  }
  fs.writeFileSync(
    path.join(dir, ".vite", "manifest.json"),
    JSON.stringify(manifest),
  );
  fs.writeFileSync(
    path.join(dir, "index.html"),
    `<script type="module" src="/assets/index-entry.js"></script>` +
      preloads
        .map((file) => `<link rel="modulepreload" href="/assets/${file}">`)
        .join(""),
  );
  return dir;
}

test("counts shared static dependencies once, including cycles and HTML preloads", (t) => {
  const dir = fixture(
    t,
    {
      react: {
        file: "vendor-react-core.js",
        bytes: 200 * KiB,
        imports: ["shared"],
      },
      shared: { file: "shared.js", bytes: 10 * KiB, imports: ["react"] },
      preload: { file: "preloaded.js", bytes: 15 * KiB, imports: ["shared"] },
    },
    ["preloaded.js"],
  );
  const report = checkBundleSize(dir);
  assert.equal(report.bootstrapBytes, 265 * KiB);
  assert.equal(report.bootstrapFiles.length, 4);
  assert.equal(report.totalBytes, 635 * KiB);
});

test("rejects oversized startup even when split across arbitrarily named small chunks", (t) => {
  const extra = {};
  for (let i = 0; i < 4; i++)
    extra[`part${i}`] = {
      file: `part${i}.js`,
      bytes: 15 * KiB,
      imports: i < 3 ? [`part${i + 1}`] : [],
    };
  extra.react = {
    file: "vendor-react-core.js",
    bytes: 200 * KiB,
    imports: ["part0"],
  };
  assert.throws(
    () => checkBundleSize(fixture(t, extra)),
    /Bootstrap JS.*300\.00 KiB/,
  );
});

test("rejects eager chart imports and chart HTML preloads", (t) => {
  const dir = fixture(t, {
    react: {
      file: "vendor-react-core.js",
      bytes: 200 * KiB,
      imports: ["charts"],
    },
  });
  assert.throws(() => checkBundleSize(dir), /Bootstrap JS/);
  assert.throws(
    () => checkBundleSize(fixture(t, {}, ["vendor-charts-library.js"])),
    /Bootstrap JS/,
  );
});

test("fails closed for missing manifest, dependency edges and output files", (t) => {
  const missingManifest = fixture(t);
  fs.unlinkSync(path.join(missingManifest, ".vite", "manifest.json"));
  assert.throws(() => checkBundleSize(missingManifest), /ENOENT/);
  assert.throws(
    () =>
      checkBundleSize(
        fixture(t, {
          react: {
            file: "vendor-react-core.js",
            bytes: 200 * KiB,
            imports: ["missing"],
          },
        }),
      ),
    /imports graph/,
  );
  assert.throws(
    () =>
      checkBundleSize(
        fixture(t, {
          route: {
            file: "ClientsPage-route.js",
            bytes: 10 * KiB,
            dynamicImports: ["missing"],
          },
        }),
      ),
    /dynamicImports graph/,
  );
  const missingChunk = fixture(t);
  fs.unlinkSync(path.join(missingChunk, "assets", "vendor-react-core.js"));
  assert.throws(() => checkBundleSize(missingChunk), /manifest chunk/);
});

test("rejects a preload omitted from the manifest and JavaScript omitted from the graph", (t) => {
  assert.throws(
    () => checkBundleSize(fixture(t, {}, ["absent.js"])),
    /HTML module missing/,
  );
  const dir = fixture(t);
  fs.writeFileSync(path.join(dir, "assets", "hidden.js"), "x");
  assert.throws(() => checkBundleSize(dir), /Untracked JavaScript/);
});

test("counts all deferred split files toward the raw total, even highly compressible data", (t) => {
  const extra = {};
  for (let i = 0; i < 7; i++)
    extra[`lazy${i}`] = { file: `nested/lazy${i}.mjs`, bytes: 20 * KiB };
  assert.throws(
    () => checkBundleSize(fixture(t, extra)),
    /Total JS.*750\.00 KiB/,
  );
});

test("preserves entry, aggregate chart and individual route limits", (t) => {
  assert.throws(
    () =>
      checkBundleSize(
        fixture(t, {
          entry: { file: "index-entry.js", bytes: 231 * KiB, isEntry: true },
        }),
      ),
    /entry JS/,
  );
  assert.throws(
    () =>
      checkBundleSize(
        fixture(t, {
          moreCharts: { file: "vendor-charts-extra.js", bytes: 31 * KiB },
        }),
      ),
    /charts JS/,
  );
  assert.throws(
    () =>
      checkBundleSize(
        fixture(t, {
          route: { file: "ClientsPage-route.js", bytes: 21 * KiB },
        }),
      ),
    /Route\/shared chunk/,
  );
});
