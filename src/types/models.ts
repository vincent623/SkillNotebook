export type AppScreen = "home" | "workspace" | "settings";

export type PackageStatus =
  | "draft"
  | "evaluating"
  | "validated"
  | "needs_eval"
  | "archived";

export type EvalOverallStatus = "usable" | "needs_improvement" | "problematic";

export type WorkspaceTab = "overview" | "files" | "preview" | "test";

export interface Workspace {
  id: string;
  name: string;
  rootPath: string;
  createdAt: string;
  updatedAt: string;
  lastOpenedAt?: string | null;
}

export interface SkillPackage {
  id: string;
  workspaceId: string;
  slug: string;
  name: string;
  description: string;
  tags: string[];
  status: PackageStatus;
  rootPath: string;
  currentVersion: number;
  lastEvalStatus?: EvalOverallStatus | null;
  relatedSkills: string[];
  bundleCandidates: string[];
  createdAt: string;
  updatedAt: string;
}

export interface PackageVersion {
  id: string;
  packageId: string;
  versionNumber: number;
  note?: string | null;
  snapshotPath: string;
  evalReportId?: string | null;
  isPinned: boolean;
  createdAt: string;
}

export interface EvalDetails {
  hasSkillMd: boolean;
  hasExamples: boolean;
  hasPrompts: boolean;
  hasScripts: boolean;
  inputDefined: boolean;
  outputDefined: boolean;
  boundariesClear: boolean;
  notes: string[];
}

export interface EvalReport {
  id: string;
  packageId: string;
  completenessScore: number;
  clarityScore: number;
  executabilityScore: number;
  overallStatus: EvalOverallStatus;
  suggestions: string[];
  details: EvalDetails;
  createdAt: string;
}

export interface PreviewModel {
  packageId: string;
  name: string;
  hasSkillMd: boolean;
  promptFiles: string[];
  exampleFiles: string[];
  referenceFiles: string[];
  scriptFiles: string[];
  testFiles: string[];
  skillMdPreview: string;
  examplePreview: string;
  finalPreview: string;
}

export interface AppBootstrap {
  workspace: Workspace;
  packages: SkillPackage[];
  evalReports: EvalReport[];
  versions: PackageVersion[];
  previews: PreviewModel[];
  selectedPackageId?: string | null;
  activityLog: string[];
}

export interface AppErrorPayload {
  code: string;
  message: string;
}

export interface AppEnvelope<T> {
  ok: boolean;
  data?: T | null;
  error?: AppErrorPayload | null;
}

export interface CreationBridgeStatus {
  mode: string;
  preferredGenerator: string;
  claudeCliAvailable: boolean;
  skillCreateCommandAvailable: boolean;
  claudeBinary: string;
  claudeModel?: string | null;
  claudeTimeoutSecs: number;
  fallbackGenerator: string;
}

export interface AppSettings {
  platform: string;
  shell: string[];
  formalVersionCap: number;
  workspaceModel: string;
  defaultWorkspaceRoot: string;
  currentWorkspaceRoot: string;
  recentWorkspaces: Workspace[];
  creationBridge: CreationBridgeStatus;
}

export interface CreatePackageFromNlRequest {
  workspaceId: string;
  prompt: string;
  context?: string | null;
}

export interface CreatePackageFromNlResponse {
  packageId: string;
  name: string;
  slug: string;
  rootPath: string;
  evalWorkspacePath: string;
  draftCreated: boolean;
  autoEvalStarted: boolean;
  validationSummary: string;
  generatorUsed: string;
  generationSummary: string;
}
