#!/usr/bin/env node

import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cargoManifest = path.join(repoRoot, "src-tauri", "Cargo.toml");
const skillBin = path.join(
  repoRoot,
  "src-tauri",
  "target",
  "debug",
  process.platform === "win32" ? "skill.exe" : "skill",
);

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

function runSkill(args) {
  return run(skillBin, args);
}

function parseJson(label, content) {
  try {
    return JSON.parse(content);
  } catch (error) {
    throw new Error(`${label} did not return valid JSON: ${error.message}\n${content}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function writeJson(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function makeProjectRoot() {
  const root = mkdtempSync(path.join(tmpdir(), "skillnotebook-e2e-root-"));
  mkdirSync(path.join(root, ".skill-notebook"), { recursive: true });
  mkdirSync(path.join(root, ".skills"), { recursive: true });
  writeJson(path.join(root, ".skill-notebook", "config.json"), {
    id: "project-root-main",
    name: "Skill Notebook E2E Root",
    createdAt: "2026-05-07T00:00:00Z",
    updatedAt: "2026-05-07T00:00:00Z",
    lastOpenedAt: "2026-05-07T00:00:00Z",
  });
  return root;
}

function makeSourceSkill() {
  const source = mkdtempSync(path.join(tmpdir(), "skillnotebook-e2e-source-"));
  mkdirSync(path.join(source, "references"), { recursive: true });
  mkdirSync(path.join(source, "examples"), { recursive: true });
  writeFileSync(
    path.join(source, "SKILL.md"),
    `---
name: E2E Notes Skill
description: "Turns raw notes into a compact reusable brief."
tags: [e2e, notes]
---

# E2E Notes Skill

Use this skill when raw notes need to become a clear reusable brief.
`,
  );
  writeFileSync(path.join(source, "references", "note.md"), "source note\n");
  return source;
}

function main() {
  run("cargo", ["build", "--quiet", "--manifest-path", cargoManifest, "--bin", "skill"]);

  const projectRoot = makeProjectRoot();
  const sourceSkill = makeSourceSkill();
  const keepTemp = process.env.E2E_KEEP_TEMP === "1";

  try {
    const importResult = parseJson(
      "skill import",
      runSkill([
        "--json",
        "--project_root",
        projectRoot,
        "import",
        sourceSkill,
        "--slug",
        "e2e-notes",
        "--no-eval",
      ]),
    );
    assert(importResult.command === "import", "expected import command");
    assert(importResult.result.packageId === "pkg-e2e-notes", "expected imported package id");
    assert(importResult.result.evalReport === null, "expected eval to be skipped");
    assert(
      existsSync(path.join(projectRoot, ".skills", "e2e-notes", "SKILL.md")),
      "imported SKILL.md was not written",
    );
    assert(
      existsSync(path.join(projectRoot, ".skills", "e2e-notes", "references", "note.md")),
      "imported reference file was not written",
    );

    const referenceResult = parseJson(
      "skill reference",
      runSkill(["--json", "--project_root", projectRoot, "reference", "pkg-e2e-notes"]),
    );
    const referenceIds = new Set(referenceResult.reference.items.map((item) => item.id));
    for (const id of [
      "package-path",
      "skill-md-path",
      "markdown-reference",
      "cli-reference",
      "global-claude-link",
      "project-claude-link",
    ]) {
      assert(referenceIds.has(id), `reference output missing ${id}`);
    }

    const draftStart = parseJson(
      "skill draft start",
      runSkill([
        "--json",
        "--project_root",
        projectRoot,
        "draft",
        "start",
        "Summarize support calls",
        "--from-file",
        path.join(sourceSkill, "references", "note.md"),
        "--agent",
        "codex",
      ]),
    );
    const draftId = draftStart.draft.draftId;
    assert(draftStart.command === "draft.start", "expected draft.start command");
    assert(draftId, "draft.start did not return a draft id");
    assert(
      existsSync(path.join(projectRoot, ".skill-notebook", "drafts", draftId, "BRIEF.md")),
      "draft BRIEF.md was not written",
    );
    assert(
      existsSync(path.join(projectRoot, ".skill-notebook", "drafts", draftId, "SKILL.md")),
      "draft SKILL.md was not written",
    );

    const draftImport = parseJson(
      "skill draft import",
      runSkill([
        "--json",
        "--project_root",
        projectRoot,
        "draft",
        "import",
        draftId,
        "--no-eval",
      ]),
    );
    assert(draftImport.command === "draft.import", "expected draft.import command");
    assert(draftImport.result.packageId.startsWith("pkg-"), "draft import missing package id");
    assert(draftImport.result.evalReport === null, "expected draft import eval to be skipped");
    assert(
      existsSync(path.join(draftImport.result.packagePath, "SKILL.md")),
      "promoted draft package was not written",
    );

    const listResult = parseJson(
      "skill find",
      runSkill(["--json", "--project_root", projectRoot, "find"]),
    );
    assert(listResult.packages.length >= 2, "expected imported packages to be discoverable");

    console.log("E2E core passed: import -> reference -> draft start -> draft import -> find");
  } finally {
    if (keepTemp) {
      console.log(`E2E_KEEP_TEMP=1; preserved project root: ${projectRoot}`);
      console.log(`E2E_KEEP_TEMP=1; preserved source skill: ${sourceSkill}`);
    } else {
      rmSync(projectRoot, { recursive: true, force: true });
      rmSync(sourceSkill, { recursive: true, force: true });
    }
  }
}

main();
