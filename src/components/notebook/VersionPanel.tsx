import { useEffect, useState } from "react";
import { getPackageVersionDiff } from "../../services/tauri-api";
import { useEditorStore } from "../../stores/editor-store";
import { useProjectStore } from "../../stores/project-store";
import type {
  EvalReport,
  PackageTestReport,
  PackageVersion,
  PackageVersionDiff,
  SkillPackage,
} from "../../types/models";
import { ScoreBar } from "../common/ScoreBar";
import { StatusBadge } from "../common/StatusBadge";
import { VersionDiffModal } from "./VersionDiffModal";

interface VersionPanelProps {
  pkg: SkillPackage;
  evalReport?: EvalReport;
  versions: PackageVersion[];
}

interface VersionSaveModalProps {
  evalReport?: EvalReport;
  isSaving: boolean;
  nextVersionNumber: number;
  note: string;
  pkg: SkillPackage;
  onCancel: () => void;
  onChangeNote: (value: string) => void;
  onConfirm: () => void;
}

interface VersionRestoreModalProps {
  isRestoring: boolean;
  version: PackageVersion;
  onCancel: () => void;
  onConfirm: () => void;
}

function avgScore(report: EvalReport): number {
  return Math.round(((report.completenessScore + report.clarityScore + report.executabilityScore) / 3) * 100);
}

function formatTimestamp(value: string | null): string {
  if (!value) return "";

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function qualityLabel(report?: EvalReport): string {
  if (!report) return "未评估";
  const score = avgScore(report);
  if (score >= 80) return "可以保存正式版本";
  if (score >= 60) return "建议补强后保存";
  return "先修复关键问题";
}

function testStatusLabel(report?: PackageTestReport | null): string {
  if (!report) return "未运行";
  if (report.status === "passed") return "测试通过";
  if (report.status === "missing") return "未配置测试";
  return "测试失败";
}

function VersionSaveModal({
  evalReport,
  isSaving,
  nextVersionNumber,
  note,
  pkg,
  onCancel,
  onChangeNote,
  onConfirm,
}: VersionSaveModalProps) {
  const noteReady = note.trim().length > 0;

  return (
    <div className="version-modal-overlay" onClick={onCancel} role="presentation">
      <div
        aria-labelledby="version-save-title"
        aria-modal="true"
        className="version-modal version-action-modal"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
      >
        <div className="version-modal-header">
          <div>
            <span className="version-modal-eyebrow">Formal Version</span>
            <h3 id="version-save-title">保存 v{nextVersionNumber}</h3>
            <p className="muted version-modal-subtitle">{pkg.slug}</p>
          </div>
          <button className="button-secondary version-modal-close" onClick={onCancel} type="button">
            关闭
          </button>
        </div>
        <div className="version-modal-body version-action-body">
          {evalReport ? (
            <div className="version-save-eval">
              <div className="version-save-eval-head">
                <span>本次评估</span>
                <strong>{avgScore(evalReport)}</strong>
              </div>
              <ScoreBar label="完整度" value={evalReport.completenessScore} />
              <ScoreBar label="清晰度" value={evalReport.clarityScore} />
              <ScoreBar label="可执行性" value={evalReport.executabilityScore} />
            </div>
          ) : null}
          <label className="field-stack">
            <span className="field-label">版本说明</span>
            <textarea
              className="version-save-textarea"
              onChange={(event) => onChangeNote(event.target.value)}
              placeholder="说明这次正式保存解决了什么、适合回滚到什么状态。"
              rows={4}
              value={note}
            />
          </label>
          <p className="version-form-hint">
            版本说明会进入快照记录。以后看 diff 或恢复版本时，它会是判断意图的锚点。
          </p>
        </div>
        <div className="version-modal-footer">
          <button className="button-secondary" disabled={isSaving} onClick={onCancel} type="button">
            取消
          </button>
          <button
            className="button-primary"
            disabled={isSaving || !noteReady}
            onClick={onConfirm}
            type="button"
          >
            {isSaving ? "保存中..." : "确认保存"}
          </button>
        </div>
      </div>
    </div>
  );
}

function VersionRestoreModal({
  isRestoring,
  version,
  onCancel,
  onConfirm,
}: VersionRestoreModalProps) {
  return (
    <div className="version-modal-overlay" onClick={onCancel} role="presentation">
      <div
        aria-labelledby="version-restore-title"
        aria-modal="true"
        className="version-modal version-action-modal"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
      >
        <div className="version-modal-header">
          <div>
            <span className="version-modal-eyebrow is-danger">Restore</span>
            <h3 id="version-restore-title">恢复到 v{version.versionNumber}</h3>
            <p className="muted version-modal-subtitle">{version.snapshotPath}</p>
          </div>
          <button className="button-secondary version-modal-close" onClick={onCancel} type="button">
            关闭
          </button>
        </div>
        <div className="version-modal-body version-action-body">
          <div className="version-restore-warning">
            当前 package 文件会被这个正式版本覆盖。打开的编辑内容会重新加载，未保存的草稿修改不会保留。
          </div>
          {version.note ? (
            <div className="version-restore-note">
              <span>版本说明</span>
              <p>{version.note}</p>
            </div>
          ) : null}
        </div>
        <div className="version-modal-footer">
          <button className="button-secondary" disabled={isRestoring} onClick={onCancel} type="button">
            取消
          </button>
          <button
            className="button-danger"
            disabled={isRestoring}
            onClick={onConfirm}
            type="button"
          >
            {isRestoring ? "恢复中..." : "确认恢复"}
          </button>
        </div>
      </div>
    </div>
  );
}

export function VersionPanel({ pkg, evalReport, versions }: VersionPanelProps) {
  const evalStatus = useProjectStore((state) => state.evalStatus);
  const evalError = useProjectStore((state) => state.evalError);
  const lastEvalPackageId = useProjectStore((state) => state.lastEvalPackageId);
  const lastEvalCreatedAt = useProjectStore((state) => state.lastEvalCreatedAt);
  const runEval = useProjectStore((state) => state.runEval);
  const testStatus = useProjectStore((state) => state.testStatus);
  const testError = useProjectStore((state) => state.testError);
  const lastTestPackageId = useProjectStore((state) => state.lastTestPackageId);
  const lastTestReport = useProjectStore((state) => state.lastTestReport);
  const runTest = useProjectStore((state) => state.runTest);
  const saveVersion = useProjectStore((state) => state.saveVersion);
  const restoreVersion = useProjectStore((state) => state.restoreVersion);
  const versionSaveStatus = useProjectStore((state) => state.versionSaveStatus);
  const versionSaveError = useProjectStore((state) => state.versionSaveError);
  const lastVersionSavedPackageId = useProjectStore((state) => state.lastVersionSavedPackageId);
  const lastVersionSavedAt = useProjectStore((state) => state.lastVersionSavedAt);
  const versionRestoreStatus = useProjectStore((state) => state.versionRestoreStatus);
  const versionRestoreError = useProjectStore((state) => state.versionRestoreError);
  const lastVersionRestoredPackageId = useProjectStore((state) => state.lastVersionRestoredPackageId);
  const lastVersionRestoredVersionId = useProjectStore((state) => state.lastVersionRestoredVersionId);
  const lastVersionRestoredVersionNumber = useProjectStore((state) => state.lastVersionRestoredVersionNumber);
  const lastVersionRestoredAt = useProjectStore((state) => state.lastVersionRestoredAt);
  const resetEditor = useEditorStore((state) => state.reset);
  const loadFileTree = useEditorStore((state) => state.loadFileTree);
  const [note, setNote] = useState("");
  const [saveOpen, setSaveOpen] = useState(false);
  const [restoreOpen, setRestoreOpen] = useState(false);
  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(versions[0]?.id ?? null);
  const [diffOpen, setDiffOpen] = useState(false);
  const [isDiffLoading, setIsDiffLoading] = useState(false);
  const [diffError, setDiffError] = useState<string | null>(null);
  const [versionDiff, setVersionDiff] = useState<PackageVersionDiff | null>(null);

  const isRunningEval = evalStatus === "submitting" && lastEvalPackageId === pkg.id;
  const isRunningTest = testStatus === "submitting" && lastTestPackageId === pkg.id;
  const isSaving = versionSaveStatus === "submitting" && lastVersionSavedPackageId === pkg.id;
  const isRestoring = versionRestoreStatus === "submitting" && lastVersionRestoredPackageId === pkg.id;
  const evalFailed = evalStatus === "error" && lastEvalPackageId === pkg.id;
  const evalSucceeded = evalStatus === "success" && lastEvalPackageId === pkg.id;
  const testRunFailed = testStatus === "error" && lastTestPackageId === pkg.id;
  const testRunSucceeded = testStatus === "success" && lastTestPackageId === pkg.id;
  const testReport = lastTestPackageId === pkg.id ? lastTestReport : null;
  const saveFailed = versionSaveStatus === "error" && lastVersionSavedPackageId === pkg.id;
  const saveSucceeded = versionSaveStatus === "success" && lastVersionSavedPackageId === pkg.id;
  const restoreFailed = versionRestoreStatus === "error" && lastVersionRestoredPackageId === pkg.id;
  const restoreSucceeded = versionRestoreStatus === "success" && lastVersionRestoredPackageId === pkg.id;
  const selectedVersion = versions.find((item) => item.id === selectedVersionId) ?? versions[0] ?? null;
  const nextVersionNumber = (versions[0]?.versionNumber ?? pkg.currentVersion) + 1;

  const checks: Array<[string, boolean]> = evalReport ? [
    ["SKILL.md", evalReport.details.hasSkillMd],
    ["示例", evalReport.details.hasExamples],
    ["提示词", evalReport.details.hasPrompts],
    ["输入", evalReport.details.inputDefined],
    ["输出", evalReport.details.outputDefined],
    ["边界", evalReport.details.boundariesClear],
  ] : [];

  useEffect(() => {
    setSelectedVersionId((current) => {
      if (current && versions.some((item) => item.id === current)) {
        return current;
      }
      return versions[0]?.id ?? null;
    });
  }, [versions]);

  useEffect(() => {
    setDiffOpen(false);
    setIsDiffLoading(false);
    setDiffError(null);
    setVersionDiff(null);
    setSaveOpen(false);
    setRestoreOpen(false);
    setNote("");
  }, [pkg.id]);

  async function handleShowDiff() {
    if (!selectedVersion) return;

    setDiffOpen(true);
    setIsDiffLoading(true);
    setDiffError(null);
    setVersionDiff(null);

    try {
      const diff = await getPackageVersionDiff(selectedVersion.id);
      setVersionDiff(diff);
    } catch (error) {
      setDiffError(error instanceof Error ? error.message : "版本差异加载失败。");
    } finally {
      setIsDiffLoading(false);
    }
  }

  async function handleSaveVersion() {
    const ok = await saveVersion(pkg.id, note.trim());
    if (ok) {
      setNote("");
      setSaveOpen(false);
    }
  }

  async function handleRestore() {
    if (!selectedVersion) return;

    const ok = await restoreVersion(selectedVersion.id, pkg.id);
    if (ok) {
      resetEditor();
      void loadFileTree(pkg.id);
      setDiffOpen(false);
      setRestoreOpen(false);
      setVersionDiff(null);
    }
  }

  return (
    <div className="version-panel">
      <div className="version-panel-head">
        <div>
          <h4 className="panel-label">质量门禁</h4>
          <strong>{qualityLabel(evalReport)}</strong>
        </div>
        {evalReport ? <StatusBadge status={evalReport.overallStatus} /> : null}
      </div>

      {evalReport ? (
        <div className="version-eval-card">
          <div className="version-score-head">
            <span>总分</span>
            <strong>{avgScore(evalReport)}</strong>
          </div>
          <ScoreBar label="完整度" value={evalReport.completenessScore} />
          <ScoreBar label="清晰度" value={evalReport.clarityScore} />
          <ScoreBar label="可执行性" value={evalReport.executabilityScore} />
          <div className="version-check-grid">
            {checks.map(([label, passed]) => (
              <span className={`version-check ${passed ? "is-pass" : "is-missing"}`} key={label}>
                {passed ? "✓" : "!"} {label}
              </span>
            ))}
          </div>
          {evalReport.suggestions.length > 0 ? (
            <ul className="version-suggestions">
              {evalReport.suggestions.slice(0, 3).map((suggestion) => (
                <li key={suggestion}>{suggestion}</li>
              ))}
            </ul>
          ) : null}
        </div>
      ) : (
        <p className="version-hint muted">
          先运行评估，确认结构完整度、清晰度和可执行性，再保存正式版本。
        </p>
      )}

      {testReport ? (
        <div className={`version-test-card is-${testReport.status}`}>
          <div className="version-test-head">
            <span>Smoke Tests</span>
            <strong>
              {testReport.status === "missing"
                ? "0/0"
                : `${testReport.passedTests}/${testReport.totalTests}`}
            </strong>
          </div>
          <p>{testStatusLabel(testReport)} · {testReport.summary}</p>
          {testReport.files[0]?.checks.length ? (
            <div className="version-test-checks">
              {testReport.files[0].checks.slice(0, 4).map((check) => (
                <span
                  className={`version-test-check ${check.passed ? "is-pass" : "is-fail"}`}
                  key={`${testReport.id}-${check.description}`}
                >
                  {check.passed ? "✓" : "!"} {check.description}
                </span>
              ))}
            </div>
          ) : null}
        </div>
      ) : null}

      <div className="version-entries">
        <div className="version-section-title">
          <span>正式版本</span>
          <strong>{versions.length}</strong>
        </div>
        {versions.length === 0 ? (
          <span className="muted version-empty">暂无版本</span>
        ) : (
          versions.map((version) => (
            <button
              className={`version-entry ${selectedVersionId === version.id ? "is-selected" : ""}`}
              key={version.id}
              onClick={() => setSelectedVersionId(version.id)}
              type="button"
            >
              <span className={`version-dot ${version.isPinned ? "is-pinned" : ""}`} />
              <span className="version-entry-body">
                <span className="version-num">v{version.versionNumber}</span>
                <span className="version-entry-note">
                  {version.note || formatTimestamp(version.createdAt)}
                </span>
              </span>
              {version.isPinned ? <span className="version-pin">已固定</span> : null}
            </button>
          ))
        )}
      </div>

      {evalFailed ? (
        <div className="inline-banner inline-banner-error">
          {evalError ?? "评估失败，请稍后重试。"}
        </div>
      ) : null}
      {evalSucceeded ? (
        <div className="inline-banner inline-banner-success">
          评估已完成{lastEvalCreatedAt ? `：${formatTimestamp(lastEvalCreatedAt)}` : "。"}
        </div>
      ) : null}
      {testRunFailed ? (
        <div className="inline-banner inline-banner-error">
          {testError ?? "测试运行失败，请稍后重试。"}
        </div>
      ) : null}
      {testRunSucceeded && testReport ? (
        <div
          className={`inline-banner ${
            testReport.status === "passed" ? "inline-banner-success" : "inline-banner-warning"
          }`}
        >
          {testStatusLabel(testReport)}：{testReport.summary}
        </div>
      ) : null}
      {saveFailed ? (
        <div className="inline-banner inline-banner-error">
          {versionSaveError ?? "版本保存失败，请稍后重试。"}
        </div>
      ) : null}
      {saveSucceeded ? (
        <div className="inline-banner inline-banner-success">
          版本已保存{lastVersionSavedAt ? `：${formatTimestamp(lastVersionSavedAt)}` : "。"}
        </div>
      ) : null}
      {restoreFailed ? (
        <div className="inline-banner inline-banner-error">
          {versionRestoreError ?? "版本恢复失败，请稍后重试。"}
        </div>
      ) : null}
      {restoreSucceeded ? (
        <div className="inline-banner inline-banner-success">
          已恢复到 {lastVersionRestoredVersionNumber ? `v${lastVersionRestoredVersionNumber}` : (lastVersionRestoredVersionId ?? "目标版本")}
          {lastVersionRestoredAt ? `：${formatTimestamp(lastVersionRestoredAt)}` : "。"}
        </div>
      ) : null}

      <div className="version-actions">
        <div className="version-action-row">
          <button
            className="button-primary version-action-btn"
            disabled={isRunningEval}
            onClick={() => { void runEval(pkg.id); }}
            type="button"
          >
            {isRunningEval ? "评估中..." : "运行评估"}
          </button>
          <button
            className="button-secondary version-action-btn"
            disabled={isRunningTest}
            onClick={() => { void runTest(pkg.id); }}
            type="button"
          >
            {isRunningTest ? "测试中..." : "运行测试"}
          </button>
        </div>
        <button
          className="button-secondary version-action-btn"
          disabled={!evalReport || isSaving}
          onClick={() => setSaveOpen(true)}
          type="button"
        >
          保存版本
        </button>
        <div className="version-action-row">
          <button
            className="button-secondary version-action-btn"
            disabled={!selectedVersion || isDiffLoading}
            onClick={() => { void handleShowDiff(); }}
            type="button"
          >
            {isDiffLoading ? "加载..." : "差异"}
          </button>
          <button
            className="button-secondary version-action-btn"
            disabled={!selectedVersion || isRestoring}
            onClick={() => setRestoreOpen(true)}
            type="button"
          >
            {isRestoring ? "恢复中..." : "恢复"}
          </button>
        </div>
      </div>

      <VersionDiffModal
        diff={versionDiff}
        errorMessage={diffError}
        isLoading={isDiffLoading}
        onClose={() => setDiffOpen(false)}
        open={diffOpen}
        version={selectedVersion}
      />
      {saveOpen ? (
        <VersionSaveModal
          evalReport={evalReport}
          isSaving={isSaving}
          nextVersionNumber={nextVersionNumber}
          note={note}
          onCancel={() => { setSaveOpen(false); setNote(""); }}
          onChangeNote={setNote}
          onConfirm={() => { void handleSaveVersion(); }}
          pkg={pkg}
        />
      ) : null}
      {restoreOpen && selectedVersion ? (
        <VersionRestoreModal
          isRestoring={isRestoring}
          onCancel={() => setRestoreOpen(false)}
          onConfirm={() => { void handleRestore(); }}
          version={selectedVersion}
        />
      ) : null}
    </div>
  );
}
