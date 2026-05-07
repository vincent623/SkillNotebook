#!/usr/bin/env node

import { createServer } from "node:net";
import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function getFreePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.unref();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      server.close(() => {
        if (typeof address === "object" && address?.port) {
          resolve(address.port);
        } else {
          reject(new Error("failed to allocate a local port"));
        }
      });
    });
  });
}

async function waitForServer(url, serverProcess) {
  const startedAt = Date.now();
  while (Date.now() - startedAt < 20_000) {
    if (serverProcess.exitCode !== null) {
      throw new Error(`Vite exited early with code ${serverProcess.exitCode}`);
    }
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Vite is still starting.
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for ${url}`);
}

async function launchBrowser() {
  const preferredChannel = process.env.E2E_BROWSER_CHANNEL || "chrome";
  try {
    return await chromium.launch({ channel: preferredChannel, headless: true });
  } catch (channelError) {
    try {
      return await chromium.launch({ headless: true });
    } catch (bundledError) {
      throw new Error(
        [
          `Failed to launch Playwright browser with channel "${preferredChannel}".`,
          "Install a browser with `npx playwright install chromium` or set E2E_BROWSER_CHANNEL to an installed channel.",
          `Channel error: ${channelError.message}`,
          `Bundled browser error: ${bundledError.message}`,
        ].join("\n"),
      );
    }
  }
}

async function main() {
  const port = await getFreePort();
  const url = `http://127.0.0.1:${port}/`;
  const server = spawn(
    "npm",
    ["run", "dev", "--", "--host", "127.0.0.1", "--port", String(port), "--strictPort"],
    {
      cwd: repoRoot,
      env: { ...process.env, BROWSER: "none" },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  const serverLog = [];
  server.stdout.on("data", (chunk) => serverLog.push(chunk.toString()));
  server.stderr.on("data", (chunk) => serverLog.push(chunk.toString()));

  let browser;
  try {
    await waitForServer(url, server);
    browser = await launchBrowser();
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    const consoleErrors = [];
    page.on("console", (message) => {
      if (message.type() === "error") consoleErrors.push(message.text());
    });

    await page.goto(url, { waitUntil: "networkidle" });
    await page.getByRole("button", { name: "导入 / 草稿" }).click();
    await page.getByRole("heading", { name: "导入或新建草稿" }).waitFor();
    await page.getByText("Skill Notebook 不在这里生成 skill").waitFor();

    await page.getByRole("button", { name: "新建草稿" }).click();
    await page.getByRole("textbox", { name: "草稿目标" }).fill("把会议纪要整理成负责人、截止日期、风险和行动项");
    await page.getByRole("button", { name: "创建草稿工作区" }).click();
    await page.getByText(/已创建草稿/).waitFor();
    await page.getByText("Draft workspace").waitFor();
    await page.getByText("交给外部 Agent").waitFor();

    await page.getByRole("button", { name: "快速引用" }).click();
    await page.getByRole("dialog", { name: "快速引用 / 使用" }).waitFor();
    await page.getByText("Quick reference").waitFor();
    await page.getByText("Markdown reference").waitFor();
    await page.getByText("CLI reference").waitFor();

    assert(consoleErrors.length === 0, `browser console errors:\n${consoleErrors.join("\n")}`);
    console.log("E2E browser passed: draft UI -> quick reference modal");
  } catch (error) {
    const logTail = serverLog.join("").split("\n").slice(-40).join("\n");
    throw new Error(`${error.message}\n\nVite log tail:\n${logTail}`);
  } finally {
    if (browser) await browser.close();
    if (server.exitCode === null) {
      server.kill("SIGTERM");
      await new Promise((resolve) => server.once("exit", resolve));
    }
  }
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
