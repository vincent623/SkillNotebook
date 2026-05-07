#!/usr/bin/env node

import { existsSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const appPath = path.join(
  repoRoot,
  "src-tauri",
  "target",
  "release",
  "bundle",
  "macos",
  "Skill Notebook.app",
);
const appBundleId = "com.zbkg.skillnotebook";
const appDisplayName = "Skill Notebook";
const executableName = "skill-notebook";

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    ...options,
  });

  if (result.status !== 0) {
    throw new Error(
      [
        `Command failed: ${command} ${args.join(" ")}`,
        result.stdout.trim(),
        result.stderr.trim(),
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }

  return result.stdout.trim();
}

function runOptional(command, args) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  return {
    ok: result.status === 0,
    stdout: result.stdout.trim(),
    stderr: result.stderr.trim(),
  };
}

function pidsForApp() {
  const result = runOptional("pgrep", ["-x", executableName]);
  if (!result.ok || !result.stdout) return new Set();
  return new Set(
    result.stdout
      .split(/\s+/)
      .map((value) => value.trim())
      .filter(Boolean),
  );
}

async function waitForNewPid(before) {
  const startedAt = Date.now();
  while (Date.now() - startedAt < 15_000) {
    const current = pidsForApp();
    const created = [...current].filter((pid) => !before.has(pid));
    if (created.length > 0) return created[0];
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for ${executableName} to launch`);
}

function windowCountForApp() {
  for (const name of [executableName, appDisplayName]) {
    const script = `tell application "System Events" to tell process "${name}" to count windows`;
    const result = runOptional("osascript", ["-e", script]);
    if (result.ok) return result;
  }
  return runOptional("osascript", [
    "-e",
    `tell application "System Events" to tell process "${executableName}" to count windows`,
  ]);
}

function quitApp(pid) {
  runOptional("osascript", ["-e", `tell application id "${appBundleId}" to quit`]);
  runOptional("osascript", ["-e", `tell application "${appDisplayName}" to quit`]);
  for (let attempt = 0; attempt < 20; attempt += 1) {
    if (!pidsForApp().has(pid)) return;
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 250);
  }
  runOptional("kill", [pid]);
}

async function main() {
  if (process.platform !== "darwin") {
    console.log("E2E native skipped: macOS bundle smoke only runs on darwin.");
    return;
  }

  if (process.env.E2E_NATIVE_SKIP_BUILD !== "1") {
    run("npm", ["run", "tauri:build"]);
  }

  if (!existsSync(appPath)) {
    throw new Error(`Native app bundle not found: ${appPath}`);
  }

  run("codesign", ["--verify", "--deep", "--strict", "--verbose=2", appPath]);

  const before = pidsForApp();
  run("open", ["-n", appPath]);
  const pid = await waitForNewPid(before);

  try {
    await new Promise((resolve) => setTimeout(resolve, 2000));
    const windowCount = windowCountForApp();
    if (windowCount.ok) {
      const count = Number(windowCount.stdout);
      if (!Number.isFinite(count) || count < 1) {
        throw new Error(`Native app launched but no window was reported. osascript output: ${windowCount.stdout}`);
      }
      console.log(`E2E native passed: bundle signed, launched pid ${pid}, window count ${count}`);
    } else {
      console.log(
        [
          `E2E native passed: bundle signed and launched pid ${pid}.`,
          "Window inspection was skipped because macOS Accessibility automation is unavailable.",
          windowCount.stderr || windowCount.stdout,
        ]
          .filter(Boolean)
          .join("\n"),
      );
    }
  } finally {
    quitApp(pid);
  }
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
