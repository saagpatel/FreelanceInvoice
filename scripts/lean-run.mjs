import { spawn } from "node:child_process";
import { execSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const commandArgs = process.argv.slice(2);

if (commandArgs.length === 0) {
  console.error("Usage: node scripts/lean-run.mjs <pnpm args>");
  process.exit(1);
}

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "freelanceinvoice-lean-"));
const viteCacheDir = path.join(tempRoot, "vite-cache");
const cargoTargetDir = path.join(tempRoot, "cargo-target");

fs.mkdirSync(viteCacheDir, { recursive: true });
fs.mkdirSync(cargoTargetDir, { recursive: true });

const env = {
  ...process.env,
  VITE_CACHE_DIR: viteCacheDir,
  CARGO_TARGET_DIR: cargoTargetDir,
};

let cleaned = false;
function safeRm(targetPath) {
  try {
    fs.rmSync(targetPath, { recursive: true, force: true });
  } catch {
    // Best-effort cleanup to avoid crashing during process teardown races.
  }
}

function cleanupLingeringVite() {
  const isTauriDev =
    commandArgs.length >= 2 && commandArgs[0] === "tauri" && commandArgs[1] === "dev";
  if (!isTauriDev) {
    return;
  }

  let pidOutput = "";
  try {
    pidOutput = execSync("lsof -nP -iTCP:1420 -sTCP:LISTEN -t", {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
  } catch {
    return;
  }

  const pids = pidOutput
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);

  for (const pid of pids) {
    try {
      const command = execSync(`ps -p ${pid} -o command=`, {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"],
      });
      if (command.includes(process.cwd()) && command.includes("vite")) {
        process.kill(Number(pid), "SIGTERM");
      }
    } catch {
      // Process already ended or cannot be inspected.
    }
  }
}

function cleanup() {
  if (cleaned) {
    return;
  }
  cleaned = true;
  cleanupLingeringVite();
  safeRm(tempRoot);
  for (const target of ["dist", "node_modules/.vite", "src-tauri/target"]) {
    safeRm(path.resolve(process.cwd(), target));
  }
}

const child = spawn("pnpm", commandArgs, {
  stdio: "inherit",
  env,
  detached: true,
});

function terminateChild(signal) {
  if (!child.pid) {
    return;
  }

  try {
    process.kill(-child.pid, signal);
  } catch {
    // Process may already be gone.
  }
}

process.on("SIGINT", () => terminateChild("SIGINT"));
process.on("SIGTERM", () => terminateChild("SIGTERM"));

child.on("exit", (code, signal) => {
  cleanup();
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});

child.on("error", (error) => {
  cleanup();
  console.error(error.message);
  process.exit(1);
});
