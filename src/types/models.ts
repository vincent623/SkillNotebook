export type AppScreen = "explorer" | "notebook" | "create" | "settings";

export type PackageStatus =
  | "draft"
  | "evaluating"
  | "validated"
  | "needs_eval"
  | "archived";

export type EvalOverallStatus = "usable" | "needs_improvement" | "problematic";

export interface ProjectRoot {
  id: string;
  name: string;
  rootPath: string;
  createdAt: string;
  updatedAt: string;
  lastOpenedAt?: string | null;
}

export interface SkillPackage {
  id: string;
  projectRootId: string;
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

export type VersionDiffChangeType = "added" | "removed" | "modified";

export interface VersionDiffEntry {
  path: string;
  changeType: VersionDiffChangeType;
  diffText: string;
}

export interface PackageVersionDiff {
  versionId: string;
  packageId: string;
  versionNumber: number;
  snapshotPath: string;
  entries: VersionDiffEntry[];
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

export type PackageTestStatus = "passed" | "failed" | "missing";

export interface PackageTestCheckResult {
  description: string;
  passed: boolean;
  evidence: string;
}

export interface PackageTestFileResult {
  path: string;
  name: string;
  passed: boolean;
  checks: PackageTestCheckResult[];
}

export interface PackageTestReport {
  id: string;
  packageId: string;
  status: PackageTestStatus;
  totalTests: number;
  passedTests: number;
  failedTests: number;
  files: PackageTestFileResult[];
  summary: string;
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

export interface FileEntry {
  path: string;
  name: string;
  isDirectory: boolean;
  children?: FileEntry[];
}

export interface FileContent {
  path: string;
  content: string;
  encoding: "utf-8";
}

export interface AppBootstrap {
  projectRoot: ProjectRoot;
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
  projectRootModel: string;
  skillRootName?: string;
  defaultProjectRoot: string;
  currentProjectRoot: string;
  recentProjectRoots: ProjectRoot[];
  creationBridge: CreationBridgeStatus;
}

export interface CreatePackageFromNlRequest {
  projectRootId: string;
  prompt: string;
  context?: string | null;
}

export interface CreatePackageFromSourcesRequest {
  projectRootId: string;
  sourcePaths: string[];
  prompt?: string | null;
  context?: string | null;
}

export interface CreatePackageFromUrlRequest {
  projectRootId: string;
  url: string;
  prompt?: string | null;
  context?: string | null;
}

export interface PackageUpdateRequest {
  name?: string | null;
  description?: string | null;
  tags?: string[] | null;
  status?: PackageStatus | null;
  relatedSkills?: string[] | null;
  bundleCandidates?: string[] | null;
}

export interface CommitPackagePreviewRequest {
  projectRootId: string;
  previewId: string;
}

export interface DiscardPackagePreviewRequest {
  projectRootId: string;
  previewId: string;
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

export interface PackagePreviewFile {
  path: string;
  content: string;
  encoding: "utf-8";
}

export interface CreatePackagePreviewResponse {
  previewId: string;
  projectRootId: string;
  name: string;
  slug: string;
  description: string;
  tags: string[];
  files: PackagePreviewFile[];
  fileTree: FileEntry[];
  generatorUsed: string;
  generationSummary: string;
  createdAt: string;
}

export interface PackageExportArtifact {
  packageId: string;
  zipPath: string;
  sizeBytes: number;
  createdAt: string;
}
