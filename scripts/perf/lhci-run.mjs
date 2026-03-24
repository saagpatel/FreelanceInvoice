#!/usr/bin/env node
import { spawn } from "node:child_process";

const isProduction = process.argv.includes("--production");
const configFile = isProduction
  ? ".lighthouserc.production.json"
  : "lighthouserc.json";

function run(cmd, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, {
      stdio: "inherit",
      shell: process.platform === "win32",
      ...options,
    });

    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`${cmd} ${args.join(" ")} exited with code ${code}`));
    });
  });
}

async function main() {
  console.log(`Running Lighthouse CI with ${configFile}`);
  await run("pnpm", ["exec", "lhci", "autorun", `--config=${configFile}`]);
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
