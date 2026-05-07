export type AppScreen = "explorer" | "notebook" | "draft" | "settings";

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

export interface AppSettings {
  platform: string;
  shell: string[];
  formalVersionCap: number;
  projectRootModel: string;
  skillRootName?: string;
  defaultProjectRoot: string;
  currentProjectRoot: string;
  settingsPath?: string | null;
  recentProjectRoots: ProjectRoot[];
  handoff: HandoffSettings;
}

export interface HandoffSettings {
  terminalCommand?: string | null;
  editorCommand?: string | null;
  agentCommand?: string | null;
  globalClaudeSkillsDir?: string | null;
  projectClaudeSkillsDirName?: string | null;
}

export interface SettingsUpdatePayload {
  handoff?: HandoffSettings;
}

export interface PackageUpdateRequest {
  name?: string | null;
  description?: string | null;
  tags?: string[] | null;
  status?: PackageStatus | null;
  relatedSkills?: string[] | null;
  bundleCandidates?: string[] | null;
}

export interface PackageExportArtifact {
  packageId: string;
  zipPath: string;
  sizeBytes: number;
  createdAt: string;
}

export type PackageReferenceItemKind = "path" | "snippet" | "command";

export interface PackageReferenceItem {
  id: string;
  label: string;
  value: string;
  kind: PackageReferenceItemKind;
}

export interface PackageReferenceResponse {
  packageId: string;
  slug: string;
  packagePath: string;
  skillMdPath: string;
  items: PackageReferenceItem[];
}

export interface PackageImportRequest {
  projectRootId: string;
  sourcePath: string;
  slug?: string | null;
  runEval?: boolean | null;
}

export interface PackageImportResponse {
  packageId: string;
  slug: string;
  packagePath: string;
  evalReport?: EvalReport | null;
  evalCommand: string;
  versionCommand: string;
  referenceCommand: string;
  importedAt: string;
}

export type DraftSourceKind = "text" | "files" | "url" | "empty";

export interface DraftStartRequest {
  projectRootId: string;
  prompt?: string | null;
  sourcePaths?: string[] | null;
  sourceUrl?: string | null;
  preferredAgentCommand?: string | null;
}

export interface DraftImportRequest {
  projectRootId: string;
  draftId: string;
  runEval?: boolean | null;
}

export interface DraftDiscardRequest {
  projectRootId: string;
  draftId: string;
}

export interface DraftWorkspace {
  draftId: string;
  projectRootId: string;
  draftPath: string;
  briefPath: string;
  intendedSlug: string;
  sourceKind: DraftSourceKind;
  sourceSummary: string;
  suggestedCommand: string;
  importCommand: string;
  createdAt: string;
}

export interface DraftImportResponse {
  draftId: string;
  packageId: string;
  slug: string;
  packagePath: string;
  evalReport?: EvalReport | null;
  evalCommand: string;
  versionCommand: string;
  referenceCommand: string;
  importedAt: string;
}
