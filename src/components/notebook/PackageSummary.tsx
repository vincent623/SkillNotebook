import { StatusBadge } from "../common/StatusBadge";
import { ScoreBar } from "../common/ScoreBar";
import type { EvalReport, SkillPackage } from "../../types/models";

interface PackageSummaryProps {
  pkg: SkillPackage;
  evalReport?: EvalReport;
}

export function PackageSummary({ pkg, evalReport }: PackageSummaryProps) {
  return (
    <div className="pkg-summary">
      <div className="pkg-summary-header">
        <h2>{pkg.name}</h2>
        <StatusBadge status={pkg.status} />
      </div>
      <p className="pkg-summary-desc">{pkg.description}</p>
      {pkg.tags.length > 0 ? (
        <div className="pkg-summary-tags">
          {pkg.tags.map((tag) => (
            <span key={tag} className="pkg-summary-tag">{tag}</span>
          ))}
        </div>
      ) : null}
      {evalReport ? (
        <div className="pkg-summary-eval">
          <ScoreBar label="完整度" value={evalReport.completenessScore} />
          <ScoreBar label="清晰度" value={evalReport.clarityScore} />
          <ScoreBar label="可执行性" value={evalReport.executabilityScore} />
          {evalReport.suggestions.length > 0 ? (
            <ul className="pkg-summary-suggestions">
              {evalReport.suggestions.map((s) => (
                <li key={s}>{s}</li>
              ))}
            </ul>
          ) : null}
        </div>
      ) : (
        <p className="muted pkg-summary-empty-eval">
          尚未评估。运行评估后再保存正式版本。
        </p>
      )}
    </div>
  );
}
