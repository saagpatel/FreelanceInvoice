#!/usr/bin/env node
import { spawn } from "node:child_process";
import fs from "node:fs";

const previewPort = 45173;
const baseUrl = `http://127.0.0.1:${previewPort}`;
const routes = [
  "/",
  "/clients",
  "/projects",
  "/time",
  "/invoices",
  "/settings",
];

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

async function waitForServer(url, previewProcess, timeoutMs = 30000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (previewProcess.exitCode !== null) {
      throw new Error("Preview server exited before becoming ready");
    }

    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {
      // Keep polling until timeout.
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  throw new Error(`Timed out waiting for preview server at ${url}`);
}

async function validateRoutes() {
  for (const route of routes) {
    const response = await fetch(`${baseUrl}${route}`);
    if (!response.ok) {
      throw new Error(
        `Route ${route} responded with status ${response.status}`,
      );
    }
    const html = await response.text();
    if (!html.includes('id="root"')) {
      throw new Error(`Route ${route} did not return app shell HTML`);
    }
  }
}

async function ensureBuild() {
  if (fs.existsSync("dist/index.html")) {
    return;
  }
  console.log("No dist build found. Building UI for e2e smoke test...");
  await run("pnpm", ["run", "build:raw"]);
}

async function main() {
  await ensureBuild();

  const preview = spawn(
    "pnpm",
    [
      "exec",
      "vite",
      "preview",
      "--strictPort",
      "--host",
      "127.0.0.1",
      "--port",
      String(previewPort),
    ],
    {
      stdio: "pipe",
      shell: process.platform === "win32",
    },
  );

  preview.stdout?.on("data", (chunk) => process.stdout.write(chunk));
  preview.stderr?.on("data", (chunk) => process.stderr.write(chunk));

  try {
    await waitForServer(`${baseUrl}/`, preview);
    await validateRoutes();
    console.log("E2E smoke checks passed.");
  } finally {
    preview.kill("SIGTERM");
  }
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
