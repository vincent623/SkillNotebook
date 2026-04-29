import type { EvalReport, SkillPackage } from "../../types/models";

const statusLabel: Record<string, string> = {
  draft: "草稿",
  evaluating: "评估中",
  validated: "已验证",
  needs_eval: "待评估",
  archived: "已归档",
};

const statusColor: Record<string, string> = {
  draft: "var(--warm)",
  evaluating: "var(--accent)",
  validated: "var(--success)",
  needs_eval: "var(--warm)",
  archived: "var(--danger)",
};

interface PackageRowProps {
  item: SkillPackage;
  evalReport?: EvalReport;
  onClick: () => void;
}

export function PackageRow({ item, evalReport, onClick }: PackageRowProps) {
  const avgScore = evalReport
    ? Math.round(
        ((evalReport.completenessScore + evalReport.clarityScore + evalReport.executabilityScore) / 3) * 100,
      )
    : null;

  return (
    <button className="pkg-row" onClick={onClick} type="button">
      <div className="pkg-row-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
      </div>
      <div className="pkg-row-body">
        <span className="pkg-row-slug">{item.slug}</span>
        <span className="pkg-row-meta">
          <span
            className="pkg-row-dot"
            style={{ background: statusColor[item.status] ?? "var(--text-tertiary)" }}
          />
          {item.name} · {statusLabel[item.status] ?? item.status} · v{item.currentVersion}
          {avgScore !== null ? (
            <span className="pkg-row-score">
              <span className="pkg-row-score-bar">
                <span
                  className="pkg-row-score-fill"
                  style={{ width: `${avgScore}%`, background: avgScore >= 80 ? "var(--accent)" : avgScore >= 60 ? "var(--warm)" : "var(--danger)" }}
                />
              </span>
              {avgScore}
            </span>
          ) : null}
        </span>
      </div>
      <svg className="pkg-row-chevron" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M9 18l6-6-6-6" />
      </svg>
    </button>
  );
}
