import fs from "node:fs";
import path from "node:path";

const mode = process.argv[2];

const HEAVY_PATHS = ["dist", "node_modules/.vite", "src-tauri/target"];
const ALL_LOCAL_PATHS = [...HEAVY_PATHS, "node_modules"];

function removePath(targetPath) {
  const resolved = path.resolve(process.cwd(), targetPath);
  const exists = fs.existsSync(resolved);
  if (!exists) {
    console.log(`skip ${targetPath} (not present)`);
    return;
  }

  fs.rmSync(resolved, { recursive: true, force: true });
  console.log(`removed ${targetPath}`);
}

if (mode !== "heavy" && mode !== "all-local") {
  console.error("Usage: node scripts/cleanup.mjs <heavy|all-local>");
  process.exit(1);
}

const targets = mode === "heavy" ? HEAVY_PATHS : ALL_LOCAL_PATHS;
for (const target of targets) {
  removePath(target);
}

