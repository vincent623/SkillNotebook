import { invoke } from "@tauri-apps/api/core";
import type {
  AppBootstrap,
  AppEnvelope,
  AppSettings,
  CommitPackagePreviewRequest,
  CreatePackageFromNlRequest,
  CreatePackageFromNlResponse,
  CreatePackageFromSourcesRequest,
  CreatePackageFromUrlRequest,
  CreatePackagePreviewResponse,
  DiscardPackagePreviewRequest,
  EvalReport,
  FileContent,
  FileEntry,
  PackageExportArtifact,
  PackagePreviewFile,
  PackageTestReport,
  PackageUpdateRequest,
  PackageVersionDiff,
  PackageVersion,
  SettingsUpdatePayload,
  SkillPackage,
  ProjectRoot,
} from "../types/models";

const demoBootstrap: AppBootstrap = {
  projectRoot: {
    id: "project-root-main",
    name: "默认技能仓库",
    rootPath: "/Users/vbot/SkillNotebook/project-root",
    createdAt: "2026-04-13T18:00:00Z",
    updatedAt: "2026-04-13T18:24:00Z",
    lastOpenedAt: "2026-04-13T18:24:00Z",
  },
  packages: [
    {
      id: "pkg-interview",
      projectRootId: "project-root-main",
      slug: "interview-insight-extractor",
      name: "访谈洞察提取器",
      description: "将零散的访谈记录转化为结构化的用户洞察与核心矛盾。",
      tags: ["研究", "综合", "定性"],
      status: "validated",
      rootPath: "/Users/vbot/SkillNotebook/project-root/.skills/interview-insight-extractor",
      currentVersion: 3,
      lastEvalStatus: "usable",
      relatedSkills: ["persona-composer"],
      bundleCandidates: ["research-pipeline"],
      createdAt: "2026-04-10T09:00:00Z",
      updatedAt: "2026-04-13T17:58:00Z",
    },
    {
      id: "pkg-pdf",
      projectRootId: "project-root-main",
      slug: "pdf-brief-builder",
      name: "PDF 简报生成器",
      description: "清洗多份 PDF 文档，生成带引用锚点的精炼简报。",
      tags: ["PDF", "清洗", "摘要"],
      status: "needs_eval",
      rootPath: "/Users/vbot/SkillNotebook/project-root/.skills/pdf-brief-builder",
      currentVersion: 1,
      lastEvalStatus: "needs_improvement",
      relatedSkills: ["report-refiner"],
      bundleCandidates: [],
      createdAt: "2026-04-11T10:20:00Z",
      updatedAt: "2026-04-13T17:51:00Z",
    },
    {
      id: "pkg-meeting",
      projectRootId: "project-root-main",
      slug: "meeting-actions-synthesizer",
      name: "会议行动提炼器",
      description: "从原始会议笔记中提取可执行的下一步行动。",
      tags: ["会议", "行动", "运营"],
      status: "draft",
      rootPath: "/Users/vbot/SkillNotebook/project-root/.skills/meeting-actions-synthesizer",
      currentVersion: 0,
      lastEvalStatus: null,
      relatedSkills: [],
      bundleCandidates: [],
      createdAt: "2026-04-13T17:42:00Z",
      updatedAt: "2026-04-13T18:12:00Z",
    },
  ],
  evalReports: [
    {
      id: "eval-interview-v3",
      packageId: "pkg-interview",
      completenessScore: 0.92,
      clarityScore: 0.88,
      executabilityScore: 0.9,
      overallStatus: "usable",
      suggestions: [
        "增加一个低信号访谈的反面示例。",
        "收紧洞察置信度的输出格式定义。",
      ],
      details: {
        hasSkillMd: true,
        hasExamples: true,
        hasPrompts: true,
        hasScripts: false,
        inputDefined: true,
        outputDefined: true,
        boundariesClear: true,
        notes: [
          "提示词指令具体且有据可依。",
          "示例输出已经展现稳定的结构。",
        ],
      },
      createdAt: "2026-04-13T17:58:00Z",
    },
    {
      id: "eval-pdf-v1",
      packageId: "pkg-pdf",
      completenessScore: 0.76,
      clarityScore: 0.69,
      executabilityScore: 0.71,
      overallStatus: "needs_improvement",
      suggestions: [
        "明确引用格式的输出约定。",
        "增加多文档合并场景的强示例。",
      ],
      details: {
        hasSkillMd: true,
        hasExamples: true,
        hasPrompts: true,
        hasScripts: true,
        inputDefined: true,
        outputDefined: false,
        boundariesClear: false,
        notes: [
          "包结构已完整。",
          "最终简报的格式定义仍不充分。",
        ],
      },
      createdAt: "2026-04-13T17:51:00Z",
    },
  ],
  versions: [
    {
      id: "version-interview-v3",
      packageId: "pkg-interview",
      versionNumber: 3,
      note: "优化综合提示词，加强证据格式化。",
      snapshotPath: ".skill-notebook/snapshots/pkg-interview/v3",
      evalReportId: "eval-interview-v3",
      isPinned: true,
      createdAt: "2026-04-13T17:59:00Z",
    },
    {
      id: "version-interview-v2",
      packageId: "pkg-interview",
      versionNumber: 2,
      note: "添加访谈模式，精炼标签。",
      snapshotPath: ".skill-notebook/snapshots/pkg-interview/v2",
      evalReportId: null,
      isPinned: false,
      createdAt: "2026-04-12T16:20:00Z",
    },
    {
      id: "version-pdf-v1",
      packageId: "pkg-pdf",
      versionNumber: 1,
      note: "首次评估后的初始正式版本。",
      snapshotPath: ".skill-notebook/snapshots/pkg-pdf/v1",
      evalReportId: "eval-pdf-v1",
      isPinned: false,
      createdAt: "2026-04-13T17:52:00Z",
    },
  ],
  previews: [
    {
      packageId: "pkg-interview",
      name: "访谈洞察提取器",
      hasSkillMd: true,
      promptFiles: ["prompts/system.md", "prompts/task.md"],
      exampleFiles: ["examples/example-01.md", "examples/example-02.md"],
      referenceFiles: ["references/interview-rubric.md"],
      scriptFiles: [],
      testFiles: ["tests/smoke-test.json"],
      skillMdPreview:
        "将访谈转化为洞察卡片，包含痛点、证据、矛盾和设计启示。",
      examplePreview:
        "洞察：当用户在三分钟内看到第一个结果时，对引导流程的信任度显著提升。",
      finalPreview:
        "导出预览包含三张洞察卡片、一份矛盾摘要和一个优先级排序的机会列表。",
    },
    {
      packageId: "pkg-pdf",
      name: "PDF 简报生成器",
      hasSkillMd: true,
      promptFiles: ["prompts/system.md"],
      exampleFiles: ["examples/example-01.md"],
      referenceFiles: ["references/citation-style.md"],
      scriptFiles: ["scripts/run.sh"],
      testFiles: ["tests/smoke-test.json"],
      skillMdPreview:
        "标准化 PDF 输入，提取关键段落，编译带引用的结构化简报。",
      examplePreview:
        "简报预览包含执行摘要、来源表格和未解决的证据缺口。",
      finalPreview:
        "当前导出预览缺少每段的稳定引用块。",
    },
    {
      packageId: "pkg-meeting",
      name: "会议行动提炼器",
      hasSkillMd: true,
      promptFiles: ["prompts/task.md"],
      exampleFiles: [],
      referenceFiles: [],
      scriptFiles: [],
      testFiles: [],
      skillMdPreview:
        "将零散的会议笔记转化为负责人、截止日期、风险和后续建议。",
      examplePreview: "暂无示例文件。",
      finalPreview: "草稿包尚未评估或版本化。",
    },
  ],
  selectedPackageId: "pkg-interview",
  activityLog: [
    "项目根目录已从本地演示模型初始化。",
    "访谈洞察提取器 v3 已固定为当前参考版本。",
    "PDF 简报生成器正在等待更明确的输出约定，再进行下次 formal save。",
  ],
};

function cloneBootstrap(): AppBootstrap {
  return JSON.parse(JSON.stringify(demoBootstrap)) as AppBootstrap;
}

function cloneSettings(): AppSettings {
  return {
    platform: "macOS",
    shell: ["zsh", "bash"],
    formalVersionCap: 10,
    projectRootModel: "local_directory",
    skillRootName: ".skills",
    defaultProjectRoot: demoBootstrap.projectRoot.rootPath,
    currentProjectRoot: demoBootstrap.projectRoot.rootPath,
    settingsPath: null,
    recentProjectRoots: [demoBootstrap.projectRoot],
    creationBridge: {
      mode: "auto",
      preferredGenerator: "template_fallback",
      piSidecarAvailable: false,
      piSidecarConfigured: false,
      piNodeBinary: "node",
      piNodeResolvedPath: null,
      piSidecarScript: null,
      piSidecarScriptPath: null,
      agentProvider: "openai-compatible",
      agentBaseUrl: null,
      agentBaseUrlConfigured: false,
      agentApiKeyConfigured: false,
      agentModel: null,
      agentTimeoutSecs: 300,
      agentRetryAttempts: 3,
      claudeCliAvailable: false,
      skillCreateCommandAvailable: false,
      claudeBinary: "claude",
      claudeModel: null,
      claudeTimeoutSecs: 300,
      claudeRetryAttempts: 3,
      claudeRetryBackoffSecs: 8,
      fallbackGenerator: "template_fallback",
    },
  };
}

function unwrapResponse<T>(response: AppEnvelope<T>): T {
  if (response.ok) {
    return response.data as T;
  }

  if (response.error) {
    throw new Error(response.error.message);
  }

  throw new Error("Unknown Tauri response");
}

function hasTauriRuntime(): boolean {
  if (typeof window === "undefined") {
    return false;
  }

  return typeof (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ === "object";
}

function runtimeRequiredError(action: string): Error {
  return new Error(
    `${action} requires the Tauri desktop runtime. Open Skill Notebook with \`npm run tauri:dev\` instead of the browser-only preview.`,
  );
}

function wrapRuntimeError(action: string, error: unknown): Error {
  if (!hasTauriRuntime()) {
    return runtimeRequiredError(action);
  }

  return error instanceof Error ? error : new Error(`${action} failed.`);
}

function demoSlugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function uniqueDemoSlug(baseSlug: string): string {
  const rootSlug = baseSlug || "new-skill";
  const existing = new Set(demoBootstrap.packages.map((item) => item.slug));
  if (!existing.has(rootSlug)) return rootSlug;

  for (let suffix = 2; suffix < 1000; suffix += 1) {
    const candidate = `${rootSlug}-${suffix}`;
    if (!existing.has(candidate)) return candidate;
  }

  return `${rootSlug}-${Date.now()}`;
}

function titleCaseSlug(slug: string): string {
  return slug
    .split("-")
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}

function summarizeDemoGoal(prompt: string) {
  const normalized = prompt.trim().replace(/\s+/g, " ").replace(/[.!?。！？]+$/g, "");
  return normalized.length > 96 ? `${normalized.slice(0, 96)}...` : normalized;
}

function sortFileTree(entries: FileEntry[]): FileEntry[] {
  return entries
    .map((entry) => ({
      ...entry,
      children: entry.children ? sortFileTree(entry.children) : entry.children,
    }))
    .sort((left, right) => {
      if (left.isDirectory !== right.isDirectory) return left.isDirectory ? -1 : 1;
      if (left.name.toLowerCase() === "skill.md") return -1;
      if (right.name.toLowerCase() === "skill.md") return 1;
      return left.name.localeCompare(right.name, "zh-CN", { sensitivity: "base" });
    });
}

function filesToTree(files: PackagePreviewFile[]): FileEntry[] {
  const root: FileEntry[] = [];

  files.forEach((file) => {
    const parts = file.path.split("/").filter(Boolean);
    let cursor = root;
    let currentPath = "";

    parts.forEach((part, index) => {
      currentPath = currentPath ? `${currentPath}/${part}` : part;
      const isDirectory = index < parts.length - 1;
      let entry = cursor.find((item) => item.path === currentPath);

      if (!entry) {
        entry = {
          path: currentPath,
          name: part,
          isDirectory,
          children: isDirectory ? [] : undefined,
        };
        cursor.push(entry);
      }

      if (isDirectory) {
        entry.children ??= [];
        cursor = entry.children;
      }
    });
  });

  return sortFileTree(root);
}

const demoCreatePreviews: Record<string, CreatePackagePreviewResponse> = {};

function makeDemoCreatePreview(
  payload: CreatePackageFromNlRequest,
): CreatePackagePreviewResponse {
  const goal = summarizeDemoGoal(payload.prompt);
  const stopWords = new Set([
    "a",
    "an",
    "and",
    "create",
    "for",
    "from",
    "into",
    "reusable",
    "skill",
    "the",
    "to",
    "turn",
    "turning",
    "with",
  ]);
  const keywords = payload.prompt
    .split(/[^A-Za-z0-9\u4e00-\u9fa5]+/)
    .map((item) => item.trim())
    .filter((item) => item.length >= 2 && !stopWords.has(item.toLowerCase()))
    .slice(0, 4);
  const englishSeed = keywords.filter((item) => /^[A-Za-z0-9]+$/.test(item)).join(" ") || "new skill";
  const slug = uniqueDemoSlug(demoSlugify(englishSeed).split("-").slice(0, 4).join("-"));
  const name = titleCaseSlug(slug);
  const description = `Structures a reusable workflow for ${goal}. Use when this task should become a repeatable skill package.`;
  const tags = keywords.length > 0 ? keywords : ["workflow", "draft"];
  const createdAt = new Date().toISOString();
  const previewId = `preview-${slug}-${Date.now()}`;
  const smokeTest = {
    name: "smoke-test",
    package: slug,
    prompt: `Use this skill for: ${goal}`,
    expectedOutput: "A structured deliverable with clear steps, output sections, and follow-up notes.",
    checks: [
      "SKILL.md frontmatter validates successfully.",
      "Prompt, example, and eval files are present.",
      "The package describes when to use the skill and what it outputs.",
    ],
  };
  const evals = {
    skill_name: slug,
    evals: [
      {
        id: 1,
        prompt: smokeTest.prompt,
        expected_output: smokeTest.expectedOutput,
        files: [],
        expectations: smokeTest.checks,
      },
    ],
  };
  const files: PackagePreviewFile[] = [
    {
      path: "SKILL.md",
      encoding: "utf-8",
      content: `---
name: ${slug}
description: "${description.replaceAll('"', '\\"')}"
metadata:
  author: skill-notebook
  version: 0.1.0
---

# ${name}

## Overview

This skill helps with ${goal} while keeping the output consistent and reusable.

## When to Use

- Use when the user asks for ${goal}.
- Use when this workflow should become a repeatable skill package.

## When Not to Use

- Do not use for unrelated requests.
- Ask for missing source material instead of guessing.

## Inputs

- primary task request
- optional supporting context or source files
- constraints that affect the final deliverable

## Outputs

- a structured deliverable for ${goal}
- concise notes about gaps, risks, or follow-up actions

## Workflow

1. Restate the goal.
2. Inspect the provided material.
3. Apply the workflow.
4. Return a structured result.

## Quick Reference

| Operation | How |
|-----------|-----|
| Draft the result | Follow \`prompts/task.md\` |
| Stay on-brief | Use \`prompts/system.md\` |

## Resources

- \`prompts/\` - Task framing.
- \`examples/\` - Sample output.
- \`evals/\` - Re-runnable expectations.
`,
    },
    {
      path: "prompts/system.md",
      encoding: "utf-8",
      content: `You are the ${name} skill. Stay within the package instructions, clarify missing inputs, and return a structured result.\n\nFocus: ${goal}.\n`,
    },
    {
      path: "prompts/task.md",
      encoding: "utf-8",
      content: `1. Confirm the goal in one sentence.\n2. Inspect the provided inputs before deciding on output shape.\n3. Produce the final deliverable for ${goal}.\n4. Call out uncertainty, missing data, and follow-up recommendations.\n`,
    },
    {
      path: "examples/example-01.md",
      encoding: "utf-8",
      content: `## Example\n\nInput: ${goal}\n\nOutput:\n- Goal: ${goal}\n- Key steps: review inputs, apply the workflow, present a structured result\n- Risks: missing context or incomplete source material\n`,
    },
    {
      path: "tests/smoke-test.json",
      encoding: "utf-8",
      content: `${JSON.stringify(smokeTest, null, 2)}\n`,
    },
    {
      path: "evals/evals.json",
      encoding: "utf-8",
      content: `${JSON.stringify(evals, null, 2)}\n`,
    },
  ];

  const preview: CreatePackagePreviewResponse = {
    previewId,
    projectRootId: payload.projectRootId,
    name,
    slug,
    description,
    tags,
    files,
    fileTree: filesToTree(files),
    generatorUsed: "template_fallback",
    generationSummary: "Browser preview used the local template generator.",
    createdAt,
  };
  demoCreatePreviews[previewId] = preview;
  return JSON.parse(JSON.stringify(preview)) as CreatePackagePreviewResponse;
}

function makeDemoCreatePreviewFromSources(
  payload: CreatePackageFromSourcesRequest,
): CreatePackagePreviewResponse {
  const sourcePaths = payload.sourcePaths.map((path) => path.trim()).filter(Boolean);
  if (sourcePaths.length === 0) {
    throw new Error("Add at least one local file or directory path.");
  }

  const inventory = [
    "# Source Inventory",
    "",
    "This browser preview cannot read local files directly. Native Tauri mode will inspect these paths and attach text excerpts when possible.",
    "",
    "## Requested Paths",
    "",
    ...sourcePaths.map((path) => `- \`${path}\``),
    "",
  ].join("\n");
  const preview = makeDemoCreatePreview({
    projectRootId: payload.projectRootId,
    prompt: payload.prompt?.trim() || `Create a reusable skill from ${sourcePaths.length} local source path(s).`,
    context: [
      payload.context?.trim(),
      "Local source paths:",
      ...sourcePaths.map((path) => `- ${path}`),
    ].filter(Boolean).join("\n"),
  });
  preview.files.push({
    path: "references/source-inventory.md",
    content: inventory,
    encoding: "utf-8",
  });
  preview.fileTree = filesToTree(preview.files);
  preview.generationSummary = `${preview.generationSummary} Source inventory attached from ${sourcePaths.length} local path(s).`;
  demoCreatePreviews[preview.previewId] = preview;

  return JSON.parse(JSON.stringify(preview)) as CreatePackagePreviewResponse;
}

function makeDemoCreatePreviewFromUrl(
  payload: CreatePackageFromUrlRequest,
): CreatePackagePreviewResponse {
  const url = payload.url.trim();
  if (!/^https?:\/\//i.test(url)) {
    throw new Error("URL must start with http:// or https://.");
  }

  const inventory = [
    "# URL Source",
    "",
    "Browser preview records the URL as source material. Native Tauri mode fetches the page and adds an excerpt when possible.",
    "",
    `- URL: ${url}`,
    "",
  ].join("\n");
  const preview = makeDemoCreatePreview({
    projectRootId: payload.projectRootId,
    prompt: payload.prompt?.trim() || `Create a reusable skill from ${url}.`,
    context: [
      payload.context?.trim(),
      `Source URL: ${url}`,
    ].filter(Boolean).join("\n"),
  });
  preview.files.push({
    path: "references/url-source.md",
    content: inventory,
    encoding: "utf-8",
  });
  preview.fileTree = filesToTree(preview.files);
  preview.generationSummary = `${preview.generationSummary} URL source attached.`;
  demoCreatePreviews[preview.previewId] = preview;

  return JSON.parse(JSON.stringify(preview)) as CreatePackagePreviewResponse;
}

function commitDemoCreatePreview(
  payload: CommitPackagePreviewRequest,
): CreatePackageFromNlResponse {
  const preview = demoCreatePreviews[payload.previewId];
  if (!preview) {
    throw new Error("Preview no longer exists. Generate it again before saving.");
  }
  if (preview.projectRootId !== payload.projectRootId) {
    throw new Error("Preview belongs to a different project root.");
  }

  const packageId = `pkg-${preview.slug}`;
  const rootPath = `${demoBootstrap.projectRoot.rootPath}/.skills/${preview.slug}`;
  const createdAt = new Date().toISOString();
  demoBootstrap.packages.unshift({
    id: packageId,
    projectRootId: preview.projectRootId,
    slug: preview.slug,
    name: preview.name,
    description: preview.description,
    tags: preview.tags,
    status: "needs_eval",
    rootPath,
    currentVersion: 0,
    lastEvalStatus: "needs_improvement",
    relatedSkills: [],
    bundleCandidates: [],
    createdAt,
    updatedAt: createdAt,
  });
  demoBootstrap.evalReports.unshift({
    id: `eval-${preview.slug}-draft`,
    packageId,
    completenessScore: 0.78,
    clarityScore: 0.74,
    executabilityScore: 0.7,
    overallStatus: "needs_improvement",
    suggestions: ["补充一个贴近真实输入的示例。", "保存正式版本前再跑一次评估。"],
    details: {
      hasSkillMd: true,
      hasExamples: true,
      hasPrompts: true,
      hasScripts: false,
      inputDefined: true,
      outputDefined: true,
      boundariesClear: false,
      notes: ["浏览器演示模式生成了完整预览文件。"],
    },
    createdAt,
  });
  demoBootstrap.previews.unshift({
    packageId,
    name: preview.name,
    hasSkillMd: true,
    promptFiles: preview.files.filter((file) => file.path.startsWith("prompts/")).map((file) => file.path),
    exampleFiles: preview.files.filter((file) => file.path.startsWith("examples/")).map((file) => file.path),
    referenceFiles: [],
    scriptFiles: [],
    testFiles: preview.files.filter((file) => file.path.startsWith("tests/")).map((file) => file.path),
    skillMdPreview: preview.files.find((file) => file.path === "SKILL.md")?.content.slice(0, 120) ?? "",
    examplePreview: preview.files.find((file) => file.path.startsWith("examples/"))?.content.slice(0, 120) ?? "",
    finalPreview: "草稿已从预览保存，等待评估与正式版本化。",
  });
  demoBootstrap.selectedPackageId = packageId;
  demoBootstrap.activityLog.unshift(`${preview.name} 已从创建预览保存为草稿。`);
  demoFileTrees[packageId] = preview.fileTree;
  preview.files.forEach((file) => {
    demoFileContents[`${packageId}/${file.path}`] = file.content;
  });
  delete demoCreatePreviews[payload.previewId];

  return {
    packageId,
    name: preview.name,
    slug: preview.slug,
    rootPath,
    evalWorkspacePath: `${demoBootstrap.projectRoot.rootPath}/.42eval/${preview.slug}`,
    draftCreated: true,
    autoEvalStarted: true,
    validationSummary: "Demo preview saved and marked for follow-up evaluation.",
    generatorUsed: preview.generatorUsed,
    generationSummary: preview.generationSummary,
  };
}

function discardDemoCreatePreview(payload: DiscardPackagePreviewRequest): boolean {
  const preview = demoCreatePreviews[payload.previewId];
  if (!preview) {
    return false;
  }
  if (preview.projectRootId !== payload.projectRootId) {
    throw new Error("Preview belongs to a different project root.");
  }

  delete demoCreatePreviews[payload.previewId];
  return true;
}

function findDemoPackage(packageId: string): SkillPackage {
  const pkg = demoBootstrap.packages.find((item) => item.id === packageId);
  if (!pkg) {
    throw new Error(`Package not found: ${packageId}`);
  }
  return pkg;
}

function updateDemoPackage(packageId: string, payload: PackageUpdateRequest): SkillPackage {
  const pkg = findDemoPackage(packageId);
  if (payload.name != null) {
    const name = payload.name.trim();
    if (!name) throw new Error("package name cannot be empty");
    pkg.name = name;
  }
  if (payload.description != null) {
    pkg.description = payload.description.trim();
  }
  if (payload.tags) {
    pkg.tags = Array.from(new Set(payload.tags.map((tag) => tag.trim()).filter(Boolean))).slice(0, 12);
  }
  if (payload.status) {
    pkg.status = payload.status;
  }
  if (payload.relatedSkills) {
    pkg.relatedSkills = Array.from(new Set(payload.relatedSkills.map((item) => item.trim()).filter(Boolean))).slice(0, 24);
  }
  if (payload.bundleCandidates) {
    pkg.bundleCandidates = Array.from(new Set(payload.bundleCandidates.map((item) => item.trim()).filter(Boolean))).slice(0, 24);
  }
  pkg.updatedAt = new Date().toISOString();
  demoBootstrap.activityLog.unshift(`${pkg.name} 元数据已更新。`);
  return JSON.parse(JSON.stringify(pkg)) as SkillPackage;
}

function makeDemoEvalReport(packageId: string): EvalReport {
  const pkg = findDemoPackage(packageId);
  const hasExamples = Boolean(demoFileTrees[packageId]?.some((entry) => entry.path === "examples"));
  const hasPrompts = Boolean(demoFileTrees[packageId]?.some((entry) => entry.path === "prompts"));
  const scoreBase = hasExamples && hasPrompts ? 0.82 : 0.68;
  const createdAt = new Date().toISOString();
  const report: EvalReport = {
    id: `eval-${pkg.slug}-${Date.now()}`,
    packageId,
    completenessScore: scoreBase,
    clarityScore: Math.max(0.62, scoreBase - 0.04),
    executabilityScore: Math.max(0.6, scoreBase - 0.06),
    overallStatus: scoreBase >= 0.8 ? "usable" : "needs_improvement",
    suggestions: scoreBase >= 0.8
      ? ["补充一个边界条件示例。", "保存正式版本前确认输出格式稳定。"]
      : ["补充 examples/example-01.md。", "完善 prompts/system.md 和输出边界。"],
    details: {
      hasSkillMd: true,
      hasExamples,
      hasPrompts,
      hasScripts: Boolean(demoFileTrees[packageId]?.some((entry) => entry.path === "scripts")),
      inputDefined: true,
      outputDefined: hasExamples,
      boundariesClear: scoreBase >= 0.8,
      notes: ["浏览器演示模式生成了本地评估结果。"],
    },
    createdAt,
  };

  demoBootstrap.evalReports = [
    report,
    ...demoBootstrap.evalReports.filter((item) => item.packageId !== packageId),
  ];
  pkg.lastEvalStatus = report.overallStatus;
  pkg.status = report.overallStatus === "usable" ? "validated" : "needs_eval";
  pkg.updatedAt = createdAt;
  demoBootstrap.activityLog.unshift(`${pkg.name} 已完成演示评估。`);
  return JSON.parse(JSON.stringify(report)) as EvalReport;
}

function demoValueHasContent(value: unknown): boolean {
  if (value == null) return false;
  if (typeof value === "string") return value.trim().length > 0;
  if (Array.isArray(value)) return value.length > 0;
  if (typeof value === "object") return Object.keys(value).length > 0;
  return true;
}

function demoPackageCorpus(packageId: string): string {
  const prefix = `${packageId}/`;
  return Object.entries(demoFileContents)
    .filter(([key]) => key.startsWith(prefix) && !key.includes("/tests/") && !key.endsWith("/notebook.json"))
    .map(([key, value]) => `${key}\n${value}`)
    .join("\n")
    .toLowerCase();
}

function demoKeywordPresent(corpus: string, keyword: string): boolean {
  if (corpus.includes(keyword)) return true;
  if (keyword.endsWith("s") && keyword.length > 4) {
    return corpus.includes(keyword.slice(0, -1));
  }
  return false;
}

function demoExpectationKeywords(value: string): string[] {
  const stopWords = new Set([
    "about", "after", "before", "check", "clear", "define", "defines", "expected", "file",
    "files", "from", "into", "output", "package", "present", "result", "should", "skill",
    "test", "that", "this", "what", "when", "with",
  ]);
  return value
    .toLowerCase()
    .split(/[^A-Za-z0-9]+/)
    .filter((item) => item.length >= 4 && !stopWords.has(item))
    .slice(0, 8);
}

function makeDemoTestReport(packageId: string): PackageTestReport {
  const pkg = findDemoPackage(packageId);
  const createdAt = new Date().toISOString();
  const prefix = `${packageId}/tests/`;
  const testEntries = Object.entries(demoFileContents)
    .filter(([key]) => key.startsWith(prefix) && key.endsWith(".json"))
    .sort(([left], [right]) => left.localeCompare(right));

  if (testEntries.length === 0) {
    return {
      id: `test-${pkg.slug}-${Date.now()}`,
      packageId,
      status: "missing",
      totalTests: 0,
      passedTests: 0,
      failedTests: 0,
      files: [],
      summary: "No smoke test JSON files found under tests/.",
      createdAt,
    };
  }

  const corpus = demoPackageCorpus(packageId);
  const tree = demoFileTrees[packageId] ?? [];
  const skillContent = demoFileContents[`${packageId}/SKILL.md`] ?? "";
  const hasPrompts = tree.some((entry) => entry.path === "prompts");
  const hasExamples = tree.some((entry) => entry.path === "examples");
  const hasEvals = tree.some((entry) => entry.path === "evals");
  const hasTests = tree.some((entry) => entry.path === "tests");
  const hasFrontmatter = skillContent.trimStart().startsWith("---");
  const inputDefined = corpus.includes("## inputs") || corpus.includes("input");
  const outputDefined = corpus.includes("## outputs") || corpus.includes("output") || corpus.includes("summary");
  const useTriggerDefined = corpus.includes("when to use") || corpus.includes("use when");
  const files = testEntries.map(([key, content]) => {
    const path = key.slice(packageId.length + 1);
    try {
      const parsed = JSON.parse(content) as {
        name?: string;
        package?: string;
        prompt?: string;
        input?: unknown;
        expectedOutput?: string;
        expected_output?: string;
        checks?: string[];
        expects?: string[];
        expectations?: string[];
      };
      const expectations = parsed.checks ?? parsed.expects ?? parsed.expectations ?? [];
      const hasInput = Boolean(parsed.prompt?.trim()) || demoValueHasContent(parsed.input);
      const hasExpected = expectations.length > 0 || Boolean(parsed.expectedOutput?.trim() ?? parsed.expected_output?.trim());
      const checks = [
        { description: "Test file parses as JSON.", passed: true, evidence: "JSON loaded successfully." },
        {
          description: "Smoke test defines input or prompt.",
          passed: hasInput,
          evidence: hasInput ? "Input material is present." : "Add a non-empty input or prompt field.",
        },
        {
          description: "Smoke test defines expectations.",
          passed: hasExpected,
          evidence: hasExpected ? "Expected output or checks are present." : "Add expectedOutput, checks, or expects.",
        },
        ...expectations.map((expectation) => {
          const lowered = expectation.toLowerCase();
          if (lowered.includes("frontmatter") || lowered.includes("validate")) {
            return {
              description: expectation,
              passed: hasFrontmatter,
              evidence: hasFrontmatter
                ? "SKILL.md starts with YAML frontmatter."
                : "SKILL.md has no YAML frontmatter block.",
            };
          }
          if (
            lowered.includes("prompt") &&
            lowered.includes("example") &&
            (lowered.includes("eval") || lowered.includes("test"))
          ) {
            const passed = hasPrompts && hasExamples && (hasEvals || hasTests);
            return {
              description: expectation,
              passed,
              evidence: `prompts=${hasPrompts}, examples=${hasExamples}, evals=${hasEvals}, tests=${hasTests}`,
            };
          }
          if (lowered.includes("when to use") || lowered.includes("use when")) {
            return {
              description: expectation,
              passed: useTriggerDefined,
              evidence: useTriggerDefined
                ? "The package describes when to use the skill."
                : "Add a Use when description or When to Use section.",
            };
          }
          if (lowered.includes("input") || lowered.includes("source")) {
            return {
              description: expectation,
              passed: inputDefined,
              evidence: inputDefined
                ? "The package defines input expectations."
                : "Add an Inputs section or input contract.",
            };
          }
          if (lowered.includes("output") || lowered.includes("deliverable") || lowered.includes("result")) {
            return {
              description: expectation,
              passed: outputDefined,
              evidence: outputDefined
                ? "The package defines output expectations."
                : "Add an Outputs section or output contract.",
            };
          }

          const keywords = demoExpectationKeywords(lowered);
          const missing = keywords.filter((keyword) => !demoKeywordPresent(corpus, keyword));
          return {
            description: expectation,
            passed: missing.length === 0,
            evidence: missing.length === 0
              ? `Matched keywords: ${keywords.join(", ")}.`
              : `Missing keywords in package content: ${missing.join(", ")}.`,
          };
        }),
      ];

      return {
        path,
        name: parsed.name?.trim() || "smoke-test",
        passed: checks.every((item) => item.passed),
        checks,
      };
    } catch (error) {
      return {
        path,
        name: "smoke-test",
        passed: false,
        checks: [{
          description: "Test file parses as JSON.",
          passed: false,
          evidence: error instanceof Error ? error.message : "Parse error.",
        }],
      };
    }
  });
  const passedTests = files.filter((file) => file.passed).length;
  const failedTests = files.length - passedTests;

  return {
    id: `test-${pkg.slug}-${Date.now()}`,
    packageId,
    status: failedTests === 0 ? "passed" : "failed",
    totalTests: files.length,
    passedTests,
    failedTests,
    files,
    summary: failedTests === 0
      ? `All ${files.length} smoke test file(s) passed.`
      : `${failedTests} of ${files.length} smoke test file(s) failed.`,
    createdAt,
  };
}

function makeDemoVersionDiff(versionId: string): PackageVersionDiff {
  const version = demoBootstrap.versions.find((item) => item.id === versionId);
  if (!version) {
    throw new Error(`Version not found: ${versionId}`);
  }
  const pkg = findDemoPackage(version.packageId);

  return {
    versionId,
    packageId: version.packageId,
    versionNumber: version.versionNumber,
    snapshotPath: version.snapshotPath,
    entries: [
      {
        path: "SKILL.md",
        changeType: "modified",
        diffText: [
          `--- ${version.snapshotPath}/SKILL.md`,
          `+++ ${pkg.rootPath}/SKILL.md`,
          "@@",
          "- # Saved formal version",
          `+ # ${pkg.name}`,
          "+ Updated draft content is compared against the saved snapshot.",
        ].join("\n"),
      },
    ],
  };
}

function saveDemoVersion(packageId: string, note?: string | null): PackageVersion {
  const pkg = findDemoPackage(packageId);
  const evalReport = demoBootstrap.evalReports.find((item) => item.packageId === packageId);
  if (!evalReport) {
    throw new Error("Run eval before saving a formal version.");
  }

  const nextVersionNumber =
    Math.max(pkg.currentVersion, ...demoBootstrap.versions
      .filter((item) => item.packageId === packageId)
      .map((item) => item.versionNumber), 0) + 1;
  const createdAt = new Date().toISOString();
  const version: PackageVersion = {
    id: `version-${pkg.slug}-v${nextVersionNumber}`,
    packageId,
    versionNumber: nextVersionNumber,
    note: note ?? null,
    snapshotPath: `.skill-notebook/snapshots/${packageId}/v${nextVersionNumber}`,
    evalReportId: evalReport.id,
    isPinned: nextVersionNumber === 1,
    createdAt,
  };

  demoBootstrap.versions.unshift(version);
  pkg.currentVersion = nextVersionNumber;
  pkg.updatedAt = createdAt;
  demoBootstrap.activityLog.unshift(`${pkg.name} 已保存 v${nextVersionNumber}。`);
  return JSON.parse(JSON.stringify(version)) as PackageVersion;
}

function restoreDemoVersion(versionId: string): SkillPackage {
  const version = demoBootstrap.versions.find((item) => item.id === versionId);
  if (!version) {
    throw new Error(`Version not found: ${versionId}`);
  }

  const pkg = findDemoPackage(version.packageId);
  pkg.currentVersion = version.versionNumber;
  pkg.updatedAt = new Date().toISOString();
  demoBootstrap.selectedPackageId = pkg.id;
  demoBootstrap.activityLog.unshift(`${pkg.name} 已恢复到 v${version.versionNumber}。`);
  return JSON.parse(JSON.stringify(pkg)) as SkillPackage;
}

export async function getAppBootstrap(): Promise<AppBootstrap> {
  try {
    const response = await invoke<AppEnvelope<AppBootstrap>>("app_bootstrap");
    return unwrapResponse(response);
  } catch (error) {
    console.warn("Falling back to local bootstrap payload.", error);
  }

  return cloneBootstrap();
}

export async function getSettings(): Promise<AppSettings> {
  try {
    const response = await invoke<AppEnvelope<AppSettings>>("settings_get");
    return unwrapResponse(response);
  } catch (error) {
    console.warn("Falling back to local settings payload.", error);
  }

  return cloneSettings();
}

export async function updateSettings(payload: SettingsUpdatePayload): Promise<AppSettings> {
  if (!hasTauriRuntime()) {
    const next = cloneSettings();
    if (payload.agentRuntime) {
      next.creationBridge.mode = payload.agentRuntime.mode ?? next.creationBridge.mode;
      next.creationBridge.agentProvider = payload.agentRuntime.provider ?? next.creationBridge.agentProvider;
      next.creationBridge.agentBaseUrl = payload.agentRuntime.baseUrl ?? null;
      next.creationBridge.agentBaseUrlConfigured = Boolean(payload.agentRuntime.baseUrl);
      next.creationBridge.agentApiKeyConfigured = Boolean(payload.agentRuntime.apiKey) && !payload.agentRuntime.clearApiKey;
      next.creationBridge.agentModel = payload.agentRuntime.model ?? null;
      next.creationBridge.piNodeBinary = payload.agentRuntime.nodeBinary ?? next.creationBridge.piNodeBinary;
      next.creationBridge.piSidecarScript = payload.agentRuntime.sidecarScript ?? null;
      next.creationBridge.agentTimeoutSecs = payload.agentRuntime.timeoutSecs ?? next.creationBridge.agentTimeoutSecs;
      next.creationBridge.agentRetryAttempts = payload.agentRuntime.retryAttempts ?? next.creationBridge.agentRetryAttempts;
    }
    return next;
  }

  try {
    const response = await invoke<AppEnvelope<AppSettings>>("settings_update", { payload });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("settings update", error);
  }
}

export async function openProjectRoot(rootPath: string): Promise<ProjectRoot> {
  if (!hasTauriRuntime()) {
    throw runtimeRequiredError("project root switching");
  }

  try {
    const response = await invoke<AppEnvelope<ProjectRoot>>("project_root_open", { rootPath });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("project root switching", error);
  }
}

export async function createProjectRoot(name: string): Promise<ProjectRoot> {
  if (!hasTauriRuntime()) {
    throw runtimeRequiredError("project root creation");
  }

  try {
    const response = await invoke<AppEnvelope<ProjectRoot>>("project_root_create", { name });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("project root creation", error);
  }
}

export async function createPackageFromNl(
  payload: CreatePackageFromNlRequest,
): Promise<CreatePackageFromNlResponse> {
  if (!hasTauriRuntime()) {
    throw runtimeRequiredError("Package creation");
  }

  try {
    const response = await invoke<AppEnvelope<CreatePackageFromNlResponse>>(
      "package_create_from_nl",
      { req: payload },
    );
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Package creation", error);
  }
}

export async function generatePackagePreviewFromNl(
  payload: CreatePackageFromNlRequest,
): Promise<CreatePackagePreviewResponse> {
  if (!hasTauriRuntime()) {
    return makeDemoCreatePreview(payload);
  }

  try {
    const response = await invoke<AppEnvelope<CreatePackagePreviewResponse>>(
      "package_generate_preview_from_nl",
      { req: payload },
    );
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Package preview generation", error);
  }
}

export async function generatePackagePreviewFromSources(
  payload: CreatePackageFromSourcesRequest,
): Promise<CreatePackagePreviewResponse> {
  if (!hasTauriRuntime()) {
    return makeDemoCreatePreviewFromSources(payload);
  }

  try {
    const response = await invoke<AppEnvelope<CreatePackagePreviewResponse>>(
      "package_generate_preview_from_sources",
      { req: payload },
    );
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Package source preview generation", error);
  }
}

export async function generatePackagePreviewFromUrl(
  payload: CreatePackageFromUrlRequest,
): Promise<CreatePackagePreviewResponse> {
  if (!hasTauriRuntime()) {
    return makeDemoCreatePreviewFromUrl(payload);
  }

  try {
    const response = await invoke<AppEnvelope<CreatePackagePreviewResponse>>(
      "package_generate_preview_from_url",
      { req: payload },
    );
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Package URL preview generation", error);
  }
}

export async function commitPackagePreview(
  payload: CommitPackagePreviewRequest,
): Promise<CreatePackageFromNlResponse> {
  if (!hasTauriRuntime()) {
    return commitDemoCreatePreview(payload);
  }

  try {
    const response = await invoke<AppEnvelope<CreatePackageFromNlResponse>>(
      "package_commit_preview",
      { req: payload },
    );
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Package preview saving", error);
  }
}

export async function discardPackagePreview(
  payload: DiscardPackagePreviewRequest,
): Promise<boolean> {
  if (!hasTauriRuntime()) {
    return discardDemoCreatePreview(payload);
  }

  try {
    const response = await invoke<AppEnvelope<boolean>>(
      "package_discard_preview",
      { req: payload },
    );
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Package preview discard", error);
  }
}

export async function updatePackage(
  packageId: string,
  payload: PackageUpdateRequest,
): Promise<SkillPackage> {
  if (!hasTauriRuntime()) {
    return updateDemoPackage(packageId, payload);
  }

  try {
    const response = await invoke<AppEnvelope<SkillPackage>>("package_update", {
      packageId,
      payload,
    });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Package updates", error);
  }
}

export async function exportPackageZip(packageId: string): Promise<PackageExportArtifact> {
  if (!hasTauriRuntime()) {
    throw runtimeRequiredError("Native package export");
  }

  try {
    const response = await invoke<AppEnvelope<PackageExportArtifact>>("package_export_zip", {
      packageId,
    });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Native package export", error);
  }
}

export async function runPackageEval(packageId: string): Promise<EvalReport> {
  if (!hasTauriRuntime()) {
    return makeDemoEvalReport(packageId);
  }

  try {
    const response = await invoke<AppEnvelope<EvalReport>>("package_run_eval", { packageId });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Eval runs", error);
  }
}

export async function runPackageTest(packageId: string): Promise<PackageTestReport> {
  if (!hasTauriRuntime()) {
    return makeDemoTestReport(packageId);
  }

  try {
    const response = await invoke<AppEnvelope<PackageTestReport>>("package_run_test", { packageId });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Package tests", error);
  }
}

export async function savePackageVersion(
  packageId: string,
  note?: string | null,
): Promise<PackageVersion> {
  if (!hasTauriRuntime()) {
    return saveDemoVersion(packageId, note);
  }

  try {
    const response = await invoke<AppEnvelope<PackageVersion>>("package_save_version", {
      packageId,
      note: note ?? null,
    });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Version saving", error);
  }
}

export async function getPackageVersionDiff(versionId: string): Promise<PackageVersionDiff> {
  if (!hasTauriRuntime()) {
    return makeDemoVersionDiff(versionId);
  }

  try {
    const response = await invoke<AppEnvelope<PackageVersionDiff>>("package_diff_version", {
      versionId,
    });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Version diffing", error);
  }
}

export async function restorePackageVersion(versionId: string): Promise<SkillPackage> {
  if (!hasTauriRuntime()) {
    return restoreDemoVersion(versionId);
  }

  try {
    const response = await invoke<AppEnvelope<SkillPackage>>("package_restore_version", {
      versionId,
    });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Version restore", error);
  }
}

/* ── Mock file system data ─────────────────────────── */

const demoFileTrees: Record<string, FileEntry[]> = {
  "pkg-interview": [
    { path: "skill.md", name: "skill.md", isDirectory: false },
    {
      path: "prompts", name: "prompts", isDirectory: true,
      children: [
        { path: "prompts/system.md", name: "system.md", isDirectory: false },
        { path: "prompts/task.md", name: "task.md", isDirectory: false },
      ],
    },
    {
      path: "examples", name: "examples", isDirectory: true,
      children: [
        { path: "examples/example-01.md", name: "example-01.md", isDirectory: false },
        { path: "examples/example-02.md", name: "example-02.md", isDirectory: false },
      ],
    },
    {
      path: "references", name: "references", isDirectory: true,
      children: [
        { path: "references/interview-rubric.md", name: "interview-rubric.md", isDirectory: false },
      ],
    },
    {
      path: "tests", name: "tests", isDirectory: true,
      children: [
        { path: "tests/smoke-test.json", name: "smoke-test.json", isDirectory: false },
      ],
    },
  ],
  "pkg-pdf": [
    { path: "skill.md", name: "skill.md", isDirectory: false },
    {
      path: "prompts", name: "prompts", isDirectory: true,
      children: [
        { path: "prompts/system.md", name: "system.md", isDirectory: false },
      ],
    },
    {
      path: "examples", name: "examples", isDirectory: true,
      children: [
        { path: "examples/example-01.md", name: "example-01.md", isDirectory: false },
      ],
    },
    {
      path: "references", name: "references", isDirectory: true,
      children: [
        { path: "references/citation-style.md", name: "citation-style.md", isDirectory: false },
      ],
    },
    {
      path: "scripts", name: "scripts", isDirectory: true,
      children: [
        { path: "scripts/run.sh", name: "run.sh", isDirectory: false },
      ],
    },
    {
      path: "tests", name: "tests", isDirectory: true,
      children: [
        { path: "tests/smoke-test.json", name: "smoke-test.json", isDirectory: false },
      ],
    },
  ],
  "pkg-meeting": [
    { path: "skill.md", name: "skill.md", isDirectory: false },
    {
      path: "prompts", name: "prompts", isDirectory: true,
      children: [
        { path: "prompts/task.md", name: "task.md", isDirectory: false },
      ],
    },
  ],
};

const demoFileContents: Record<string, string> = {
  "pkg-interview/skill.md": `# 访谈洞察提取器

将访谈转化为洞察卡片，包含痛点、证据、矛盾和设计启示。

## 输入

- 原始访谈记录（文本格式）
- 访谈对象的角色描述（可选）

## 输出

- 结构化洞察卡片（JSON 或 Markdown）
- 每张卡片包含：洞察标题、证据引用、置信度、矛盾点

## 边界

- 不做情感分析
- 不做自动分类（由用户标注标签）
- 单次处理一份访谈记录
`,
  "pkg-interview/prompts/system.md": `你是一个用户研究专家，擅长从访谈记录中提取结构化的用户洞察。

你的任务是将零散的访谈内容转化为可执行的洞察卡片。每张卡片必须包含：
1. 洞察标题（一句话概括）
2. 原文证据（直接引用访谈内容）
3. 置信度（高/中/低）
4. 与其他洞察的矛盾点（如果有）

保持客观，不添加个人解读。
`,
  "pkg-interview/prompts/task.md": `请分析以下访谈记录，提取关键洞察。

## 访谈记录

{{input}}

## 要求

1. 提取 3-7 个核心洞察
2. 每个洞察用卡片格式输出
3. 标注证据的原文位置
4. 识别洞察之间的矛盾
5. 给出整体置信度评估
`,
  "pkg-interview/examples/example-01.md": `# 示例 1：产品经理访谈

## 输入

> "我们的用户其实不太看数据报表，他们更关心的是异常提醒。但是老板每周都要看报表，所以我们还是得做..."

## 输出

### 洞察 1
- **标题**: 用户关注异常而非常规报表
- **证据**: "用户其实不太看数据报表，他们更关心的是异常提醒"
- **置信度**: 高
- **矛盾**: 与管理层需求冲突（见洞察 2）

### 洞察 2
- **标题**: 管理层驱动的报表需求与用户实际需求脱节
- **证据**: "老板每周都要看报表，所以我们还是得做"
- **置信度**: 中
- **矛盾**: 与洞察 1 形成需求张力
`,
  "pkg-interview/examples/example-02.md": `# 示例 2：低信号访谈

## 输入

> "还行吧，产品用着还可以。没什么特别的。"

## 输出

### 洞察 1
- **标题**: 用户无显著痛点或惊喜点
- **证据**: "还行吧，产品用着还可以"
- **置信度**: 低
- **备注**: 低信号访谈，建议追问具体使用场景
`,
  "pkg-interview/references/interview-rubric.md": `# 访谈洞察评估标准

| 维度 | 高质量 | 低质量 |
|------|--------|--------|
| 证据 | 直接引用原文 | 模糊转述 |
| 置信度 | 基于多处佐证 | 单一来源 |
| 矛盾识别 | 主动标注冲突 | 忽略矛盾 |
| 可执行性 | 指向明确行动 | 仅描述现象 |
`,
  "pkg-interview/tests/smoke-test.json": `{
  "name": "smoke-test",
  "input": "用户说他每天花 30 分钟在搜索上，但其实只找到有用内容 2-3 次。",
  "expectedOutputContains": ["洞察", "证据", "置信度"]
}
`,
  "pkg-pdf/skill.md": `# PDF 简报生成器

标准化 PDF 输入，提取关键段落，编译带引用的结构化简报。

## 输入

- 一到多份 PDF 文档

## 输出

- 带引用锚点的精炼简报（Markdown 格式）

## 已知问题

- 最终简报的格式定义仍不充分
- 缺少每段的稳定引用块
`,
  "pkg-pdf/prompts/system.md": `你是一个文档分析专家。你的任务是从 PDF 文档中提取关键信息，生成结构化简报。

输出格式：
1. 执行摘要（3-5 句）
2. 关键发现（带页码引用）
3. 未解决的问题
`,
  "pkg-pdf/examples/example-01.md": `# 简报示例

## 执行摘要
本报告分析了 Q3 用户留存数据，发现核心流失节点在第 7 天...

## 关键发现
- 第 7 天留存率下降 23%（来源：第 12 页）
- 推送通知打开率与留存正相关（来源：第 18 页）
`,
  "pkg-pdf/references/citation-style.md": `# 引用格式规范

所有引用需包含：
- 来源文档名
- 页码
- 段落位置（上/中/下）
`,
  "pkg-pdf/scripts/run.sh": `#!/bin/bash
echo "Running PDF brief builder..."
python3 scripts/extract.py "$1"
`,
  "pkg-pdf/tests/smoke-test.json": `{
  "name": "pdf-smoke",
  "input": "sample.pdf",
  "expectedOutputContains": ["执行摘要", "关键发现"]
}
`,
  "pkg-meeting/skill.md": `# 会议行动提炼器

将零散的会议笔记转化为负责人、截止日期、风险和后续建议。

## 输入

- 原始会议笔记（文本）

## 输出

- 行动项列表（负责人 + 截止日期 + 优先级）
- 风险清单
- 后续会议建议
`,
  "pkg-meeting/prompts/task.md": `请分析以下会议笔记，提取可执行的行动项。

## 会议笔记

{{input}}

## 输出要求

1. 每个行动项包含：描述、负责人、截止日期、优先级
2. 列出讨论中提到的风险
3. 建议下次会议的议题
`,
};

export async function getPackageFileTree(packageId: string): Promise<FileEntry[]> {
  if (hasTauriRuntime()) {
    try {
      const response = await invoke<AppEnvelope<FileEntry[]>>("package_file_tree", { packageId });
      return unwrapResponse(response);
    } catch (error) {
      throw wrapRuntimeError("File tree loading", error);
    }
  }
  return JSON.parse(JSON.stringify(demoFileTrees[packageId] ?? []));
}

export async function readPackageFile(packageId: string, path: string): Promise<FileContent> {
  if (hasTauriRuntime()) {
    try {
      const response = await invoke<AppEnvelope<FileContent>>("package_file_read", { packageId, path });
      return unwrapResponse(response);
    } catch (error) {
      throw wrapRuntimeError("File reading", error);
    }
  }
  const key = `${packageId}/${path}`;
  return { path, content: demoFileContents[key] ?? `（暂无 ${path} 的演示内容）`, encoding: "utf-8" };
}

export async function writePackageFile(packageId: string, path: string, content: string): Promise<void> {
  if (hasTauriRuntime()) {
    try {
      const response = await invoke<AppEnvelope<null>>("package_file_write", { packageId, path, content });
      unwrapResponse(response);
      return;
    } catch (error) {
      throw wrapRuntimeError("File writing", error);
    }
  }
  // Demo mode: update in-memory content
  demoFileContents[`${packageId}/${path}`] = content;
}
