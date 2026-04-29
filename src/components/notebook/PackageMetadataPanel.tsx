import { useMemo, useState } from "react";
import { updatePackage } from "../../services/tauri-api";
import { useProjectStore } from "../../stores/project-store";
import type { EvalReport, PackageStatus, SkillPackage } from "../../types/models";
import { ScoreBar } from "../common/ScoreBar";
import { StatusBadge } from "../common/StatusBadge";

interface PackageMetadataPanelProps {
  pkg: SkillPackage;
  evalReport?: EvalReport;
}

const statusOptions: Array<{ value: PackageStatus; label: string }> = [
  { value: "draft", label: "草稿" },
  { value: "needs_eval", label: "待评估" },
  { value: "validated", label: "已验证" },
  { value: "evaluating", label: "评估中" },
  { value: "archived", label: "已归档" },
];

function splitList(value: string) {
  return value
    .split(/[,，\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function joinList(value: string[]) {
  return value.join(", ");
}

function avgScore(report: EvalReport) {
  return Math.round(((report.completenessScore + report.clarityScore + report.executabilityScore) / 3) * 100);
}

export function PackageMetadataPanel(props: PackageMetadataPanelProps) {
  return <PackageMetadataForm key={`${props.pkg.id}-${props.pkg.updatedAt}`} {...props} />;
}

function PackageMetadataForm({ pkg, evalReport }: PackageMetadataPanelProps) {
  const refreshBootstrap = useProjectStore((state) => state.refreshBootstrap);
  const [name, setName] = useState(pkg.name);
  const [description, setDescription] = useState(pkg.description);
  const [tags, setTags] = useState(joinList(pkg.tags));
  const [status, setStatus] = useState<PackageStatus>(pkg.status);
  const [relatedSkills, setRelatedSkills] = useState(joinList(pkg.relatedSkills));
  const [bundleCandidates, setBundleCandidates] = useState(joinList(pkg.bundleCandidates));
  const [saveState, setSaveState] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const dirty = useMemo(
    () =>
      name !== pkg.name ||
      description !== pkg.description ||
      tags !== joinList(pkg.tags) ||
      status !== pkg.status ||
      relatedSkills !== joinList(pkg.relatedSkills) ||
      bundleCandidates !== joinList(pkg.bundleCandidates),
    [bundleCandidates, description, name, pkg, relatedSkills, status, tags],
  );

  async function handleSave() {
    setSaveState("saving");
    setErrorMessage(null);
    try {
      await updatePackage(pkg.id, {
        name,
        description,
        tags: splitList(tags),
        status,
        relatedSkills: splitList(relatedSkills),
        bundleCandidates: splitList(bundleCandidates),
      });
      await refreshBootstrap(pkg.id);
      setSaveState("saved");
    } catch (error) {
      setSaveState("error");
      setErrorMessage(error instanceof Error ? error.message : "元数据保存失败。");
    }
  }

  return (
    <div className="metadata-panel">
      <header className="metadata-panel-header">
        <div>
          <span className="field-label">Metadata</span>
          <h2>{pkg.name}</h2>
          <p>{pkg.slug} · v{pkg.currentVersion}</p>
        </div>
        <StatusBadge status={pkg.status} />
      </header>

      {evalReport ? (
        <section className="metadata-quality">
          <div className="metadata-score">
            <span>最新评估</span>
            <strong>{avgScore(evalReport)}</strong>
          </div>
          <ScoreBar label="完整度" value={evalReport.completenessScore} />
          <ScoreBar label="清晰度" value={evalReport.clarityScore} />
          <ScoreBar label="可执行性" value={evalReport.executabilityScore} />
        </section>
      ) : (
        <div className="inline-banner inline-banner-warning">尚未评估。保存正式版本前需要先运行评估。</div>
      )}

      <div className="metadata-form">
        <label className="field-stack">
          <span className="field-label">名称</span>
          <input className="detail-save-input" onChange={(event) => setName(event.target.value)} value={name} />
        </label>
        <label className="field-stack">
          <span className="field-label">描述</span>
          <textarea
            className="form-textarea form-textarea-sm"
            onChange={(event) => setDescription(event.target.value)}
            rows={3}
            value={description}
          />
        </label>
        <label className="field-stack">
          <span className="field-label">状态</span>
          <select className="detail-save-input" onChange={(event) => setStatus(event.target.value as PackageStatus)} value={status}>
            {statusOptions.map((option) => (
              <option key={option.value} value={option.value}>{option.label}</option>
            ))}
          </select>
        </label>
        <label className="field-stack">
          <span className="field-label">标签</span>
          <input className="detail-save-input" onChange={(event) => setTags(event.target.value)} value={tags} />
        </label>
        <label className="field-stack">
          <span className="field-label">相关技能</span>
          <input className="detail-save-input" onChange={(event) => setRelatedSkills(event.target.value)} value={relatedSkills} />
        </label>
        <label className="field-stack">
          <span className="field-label">Bundle 候选</span>
          <input className="detail-save-input" onChange={(event) => setBundleCandidates(event.target.value)} value={bundleCandidates} />
        </label>
      </div>

      {errorMessage ? <div className="inline-banner inline-banner-error">{errorMessage}</div> : null}
      {saveState === "saved" ? <div className="inline-banner inline-banner-success">元数据已保存。</div> : null}

      <div className="metadata-actions">
        <button
          className="button-primary"
          disabled={!dirty || saveState === "saving"}
          onClick={() => { void handleSave(); }}
          type="button"
        >
          {saveState === "saving" ? "保存中..." : "保存元数据"}
        </button>
      </div>
    </div>
  );
}
