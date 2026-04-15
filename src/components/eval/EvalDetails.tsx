import type { EvalReport } from "../../types/models";

interface EvalDetailsProps {
  report?: EvalReport;
}

function EvalFlag({ label, value }: { label: string; value: boolean }) {
  return (
    <div className="flag-row">
      <span>{label}</span>
      <strong>{value ? "Yes" : "No"}</strong>
    </div>
  );
}

export function EvalDetails({ report }: EvalDetailsProps) {
  if (!report) {
    return null;
  }

  return (
    <div className="content-card">
      <div className="section-head">
        <h3>Eval details</h3>
        <span className="section-meta">{report.createdAt.slice(0, 10)}</span>
      </div>
      <div className="flag-grid">
        <EvalFlag
          label="Has SKILL.md"
          value={report.details.hasSkillMd}
        />
        <EvalFlag
          label="Has examples"
          value={report.details.hasExamples}
        />
        <EvalFlag
          label="Has prompts"
          value={report.details.hasPrompts}
        />
        <EvalFlag
          label="Has scripts"
          value={report.details.hasScripts}
        />
        <EvalFlag
          label="Input defined"
          value={report.details.inputDefined}
        />
        <EvalFlag
          label="Output defined"
          value={report.details.outputDefined}
        />
        <EvalFlag
          label="Boundaries clear"
          value={report.details.boundariesClear}
        />
      </div>
      <ul className="detail-list compact-list">
        {report.suggestions.map((suggestion) => (
          <li key={suggestion}>{suggestion}</li>
        ))}
      </ul>
    </div>
  );
}
