interface ScoreBarProps {
  label: string;
  value: number;
}

function scoreTone(value: number): string {
  if (value >= 0.8) return "score-high";
  if (value >= 0.6) return "score-mid";
  return "score-low";
}

export function ScoreBar({ label, value }: ScoreBarProps) {
  const pct = Math.round(value * 100);

  return (
    <div className="score-bar-wrap">
      <div className="score-bar-header">
        <span className="score-bar-label">{label}</span>
        <span className="score-bar-value">{pct}%</span>
      </div>
      <div className="score-bar-track">
        <div
          className={`score-bar-fill ${scoreTone(value)}`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
