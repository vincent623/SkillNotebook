import { invoke } from "@tauri-apps/api/core";
import type {
  AppBootstrap,
  AppEnvelope,
  AppSettings,
  CreatePackageFromNlRequest,
  CreatePackageFromNlResponse,
  EvalReport,
  PackageVersion,
  Workspace,
} from "../types/models";

const demoBootstrap: AppBootstrap = {
  workspace: {
    id: "workspace-main",
    name: "Skill Notebook Workspace",
    rootPath: "/Users/vbot/SkillNotebook/workspace",
    createdAt: "2026-04-13T18:00:00Z",
    updatedAt: "2026-04-13T18:24:00Z",
    lastOpenedAt: "2026-04-13T18:24:00Z",
  },
  packages: [
    {
      id: "pkg-interview",
      workspaceId: "workspace-main",
      slug: "interview-insight-extractor",
      name: "Interview Insight Extractor",
      description: "Turn messy interview transcripts into structured user insights and tensions.",
      tags: ["research", "synthesis", "qualitative"],
      status: "validated",
      rootPath: "/Users/vbot/SkillNotebook/workspace/packages/interview-insight-extractor",
      currentVersion: 3,
      lastEvalStatus: "usable",
      relatedSkills: ["persona-composer"],
      bundleCandidates: ["research-pipeline"],
      createdAt: "2026-04-10T09:00:00Z",
      updatedAt: "2026-04-13T17:58:00Z",
    },
    {
      id: "pkg-pdf",
      workspaceId: "workspace-main",
      slug: "pdf-brief-builder",
      name: "PDF Brief Builder",
      description: "Clean a pile of PDFs and turn them into a concise brief with source anchors.",
      tags: ["pdf", "cleanup", "summary"],
      status: "needs_eval",
      rootPath: "/Users/vbot/SkillNotebook/workspace/packages/pdf-brief-builder",
      currentVersion: 1,
      lastEvalStatus: "needs_improvement",
      relatedSkills: ["report-refiner"],
      bundleCandidates: [],
      createdAt: "2026-04-11T10:20:00Z",
      updatedAt: "2026-04-13T17:51:00Z",
    },
    {
      id: "pkg-meeting",
      workspaceId: "workspace-main",
      slug: "meeting-actions-synthesizer",
      name: "Meeting Actions Synthesizer",
      description: "Draft action-oriented next steps from raw meeting notes and fragments.",
      tags: ["meeting", "actions", "ops"],
      status: "draft",
      rootPath: "/Users/vbot/SkillNotebook/workspace/packages/meeting-actions-synthesizer",
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
        "Add one negative example for low-signal interviews.",
        "Tighten the expected output schema for insight confidence.",
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
          "Prompt instructions are specific and grounded.",
          "Example outputs already demonstrate a stable structure.",
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
        "Clarify the output contract for citation formatting.",
        "Add a stronger example for multi-document merge behavior.",
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
          "The package is structurally complete.",
          "The final brief schema is still underspecified.",
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
      note: "Sharper synthesis prompt and stronger evidence formatting.",
      snapshotPath: ".skill-notebook/snapshots/pkg-interview/v3",
      evalReportId: "eval-interview-v3",
      isPinned: true,
      createdAt: "2026-04-13T17:59:00Z",
    },
    {
      id: "version-interview-v2",
      packageId: "pkg-interview",
      versionNumber: 2,
      note: "Added interview patterns and refined tags.",
      snapshotPath: ".skill-notebook/snapshots/pkg-interview/v2",
      evalReportId: null,
      isPinned: false,
      createdAt: "2026-04-12T16:20:00Z",
    },
    {
      id: "version-pdf-v1",
      packageId: "pkg-pdf",
      versionNumber: 1,
      note: "Initial formal version after first eval pass.",
      snapshotPath: ".skill-notebook/snapshots/pkg-pdf/v1",
      evalReportId: "eval-pdf-v1",
      isPinned: false,
      createdAt: "2026-04-13T17:52:00Z",
    },
  ],
  previews: [
    {
      packageId: "pkg-interview",
      name: "Interview Insight Extractor",
      hasSkillMd: true,
      promptFiles: ["prompts/system.md", "prompts/task.md"],
      exampleFiles: ["examples/example-01.md", "examples/example-02.md"],
      referenceFiles: ["references/interview-rubric.md"],
      scriptFiles: [],
      testFiles: ["tests/smoke-test.json"],
      skillMdPreview:
        "Transform interviews into insight cards with pain point, evidence, tension, and design implication.",
      examplePreview:
        "Insight: users trust onboarding when the first outcome appears in under three minutes.",
      finalPreview:
        "Export preview shows three insight cards, one tension summary, and a prioritized opportunity list.",
    },
    {
      packageId: "pkg-pdf",
      name: "PDF Brief Builder",
      hasSkillMd: true,
      promptFiles: ["prompts/system.md"],
      exampleFiles: ["examples/example-01.md"],
      referenceFiles: ["references/citation-style.md"],
      scriptFiles: ["scripts/run.sh"],
      testFiles: ["tests/smoke-test.json"],
      skillMdPreview:
        "Normalize PDF inputs, extract key sections, and compile a structured brief with citations.",
      examplePreview:
        "Brief preview contains executive summary, source table, and unresolved evidence gaps.",
      finalPreview:
        "Current export preview is missing a stable citation block for every paragraph.",
    },
    {
      packageId: "pkg-meeting",
      name: "Meeting Actions Synthesizer",
      hasSkillMd: true,
      promptFiles: ["prompts/task.md"],
      exampleFiles: [],
      referenceFiles: [],
      scriptFiles: [],
      testFiles: [],
      skillMdPreview:
        "Turn scattered meeting notes into owners, deadlines, risks, and follow-up suggestions.",
      examplePreview: "No example files yet.",
      finalPreview: "Draft package has not been evaluated or versioned yet.",
    },
  ],
  selectedPackageId: "pkg-interview",
  activityLog: [
    "Workspace bootstrapped from a local demo model.",
    "Interview Insight Extractor v3 is pinned as the current reference version.",
    "PDF Brief Builder is waiting for a stronger output contract before the next save.",
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
    workspaceModel: "local_directory",
    defaultWorkspaceRoot: demoBootstrap.workspace.rootPath,
    currentWorkspaceRoot: demoBootstrap.workspace.rootPath,
    recentWorkspaces: [demoBootstrap.workspace],
    creationBridge: {
      mode: "auto",
      preferredGenerator: "template_fallback",
      claudeCliAvailable: false,
      skillCreateCommandAvailable: false,
      claudeBinary: "claude",
      claudeModel: null,
      claudeTimeoutSecs: 60,
      fallbackGenerator: "template_fallback",
    },
  };
}

function unwrapResponse<T>(response: AppEnvelope<T>): T {
  if (response.ok && response.data) {
    return response.data;
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

export async function openWorkspace(rootPath: string): Promise<Workspace> {
  if (!hasTauriRuntime()) {
    throw runtimeRequiredError("Workspace switching");
  }

  try {
    const response = await invoke<AppEnvelope<Workspace>>("workspace_open", { rootPath });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Workspace switching", error);
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

export async function runPackageEval(packageId: string): Promise<EvalReport> {
  if (!hasTauriRuntime()) {
    throw runtimeRequiredError("Eval runs");
  }

  try {
    const response = await invoke<AppEnvelope<EvalReport>>("package_run_eval", { packageId });
    return unwrapResponse(response);
  } catch (error) {
    throw wrapRuntimeError("Eval runs", error);
  }
}

export async function savePackageVersion(
  packageId: string,
  note?: string | null,
): Promise<PackageVersion> {
  if (!hasTauriRuntime()) {
    throw runtimeRequiredError("Version saving");
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
