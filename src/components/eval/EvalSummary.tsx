import { StatusBadge } from "../common/StatusBadge";
import type { EvalReport } from "../../types/models";

interface EvalSummaryProps {
  packageId: string;
  report?: EvalReport;
  isRunning: boolean;
  errorMessage?: string | null;
  lastRanAt?: string | null;
  onRunEval: (packageId: string) => Promise<boolean>;
}

function formatEvalTime(value?: string | null) {
  if (!value) {
    return null;
  }

  return value.replace("T", " ").slice(0, 16);
}

export function EvalSummary({
  packageId,
  report,
  isRunning,
  errorMessage,
  lastRanAt,
  onRunEval,
}: EvalSummaryProps) {
  if (!report) {
    return (
      <div className="content-card">
        <div className="section-head">
          <h3>Eval</h3>
          <span className="section-meta">Not yet run</span>
        </div>
        <p className="muted">Run eval after drafting or editing the package to unlock formal save.</p>
        <div className="button-row">
          <button
            className="button-secondary"
            disabled={isRunning}
            onClick={() => {
              void onRunEval(packageId);
            }}
            type="button"
          >
            {isRunning ? "Running Eval..." : "Run Eval"}
          </button>
        </div>
        {errorMessage ? <div className="inline-banner inline-banner-error">{errorMessage}</div> : null}
      </div>
    );
  }

  return (
    <div className="content-card">
      <div className="section-head">
        <h3>Eval</h3>
        <StatusBadge status={report.overallStatus} />
      </div>
      <div className="button-row">
        <button
          className="button-secondary"
          disabled={isRunning}
          onClick={() => {
            void onRunEval(packageId);
          }}
          type="button"
        >
          {isRunning ? "Running Eval..." : "Re-run Eval"}
        </button>
      </div>
      {lastRanAt ? (
        <div className="inline-banner inline-banner-success">
          Eval refreshed from local package files at {formatEvalTime(lastRanAt)}.
        </div>
      ) : null}
      {errorMessage ? <div className="inline-banner inline-banner-error">{errorMessage}</div> : null}
      <div className="metric-grid single-column">
        <div className="metric-card">
          <span className="metric-label">Completeness</span>
          <strong>{Math.round(report.completenessScore * 100)}%</strong>
        </div>
        <div className="metric-card">
          <span className="metric-label">Clarity</span>
          <strong>{Math.round(report.clarityScore * 100)}%</strong>
        </div>
        <div className="metric-card">
          <span className="metric-label">Executability</span>
          <strong>{Math.round(report.executabilityScore * 100)}%</strong>
        </div>
      </div>
    </div>
  );
}
