import { StatusBadge } from "../common/StatusBadge";
import type { EvalReport, SkillPackage } from "../../types/models";

interface PackageOverviewProps {
  item: SkillPackage;
  evalReport?: EvalReport;
}

export function PackageOverview({ item, evalReport }: PackageOverviewProps) {
  return (
    <div className="panel-scroll">
      <section className="content-card hero-card compact">
        <div className="hero-header">
          <div>
            <p className="eyebrow">Skill package</p>
            <h2>{item.name}</h2>
          </div>
          <StatusBadge status={item.status} />
        </div>
        <p className="body-copy">{item.description}</p>
        <div className="metric-grid">
          <div className="metric-card">
            <span className="metric-label">Formal version</span>
            <strong>v{item.currentVersion}</strong>
          </div>
          <div className="metric-card">
            <span className="metric-label">Last eval</span>
            <strong>{item.lastEvalStatus ? item.lastEvalStatus.replace("_", " ") : "not run"}</strong>
          </div>
          <div className="metric-card">
            <span className="metric-label">Tags</span>
            <strong>{item.tags.length}</strong>
          </div>
        </div>
      </section>

      <section className="content-card">
        <div className="section-head">
          <h3>Why this package exists</h3>
          <span className="section-meta">Notebook + IDE + version log</span>
        </div>
        <ul className="detail-list">
          <li>This scaffold treats a skill as a package directory, not a single markdown file.</li>
          <li>Formal versions are intended to follow an eval pass, not every casual edit.</li>
          <li>The center panel will eventually host editing, preview, and CLI test flows.</li>
        </ul>
      </section>

      {evalReport ? (
        <section className="content-card">
          <div className="section-head">
            <h3>Current evaluation posture</h3>
            <span className="section-meta">Latest pass</span>
          </div>
          <div className="metric-grid">
            <div className="metric-card">
              <span className="metric-label">Completeness</span>
              <strong>{Math.round(evalReport.completenessScore * 100)}%</strong>
            </div>
            <div className="metric-card">
              <span className="metric-label">Clarity</span>
              <strong>{Math.round(evalReport.clarityScore * 100)}%</strong>
            </div>
            <div className="metric-card">
              <span className="metric-label">Executability</span>
              <strong>{Math.round(evalReport.executabilityScore * 100)}%</strong>
            </div>
          </div>
        </section>
      ) : null}
    </div>
  );
}
