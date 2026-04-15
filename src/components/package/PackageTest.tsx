import type { SkillPackage } from "../../types/models";

interface PackageTestProps {
  item: SkillPackage;
  activityLog: string[];
}

export function PackageTest({ item, activityLog }: PackageTestProps) {
  return (
    <div className="panel-scroll">
      <section className="content-card">
        <div className="section-head">
          <h3>CLI test surface</h3>
          <span className="section-meta">macOS shell first</span>
        </div>
        <div className="command-box">
          <code>python3 docs/skill-creator/scripts/quick_validate.py "{item.rootPath}"</code>
        </div>
        <p className="muted">
          Rust structural eval now mirrors artifacts into <code>.42eval/{item.slug}</code>, while
          the validation helper remains available as an explicit shell check.
        </p>
      </section>

      <section className="content-card">
        <div className="section-head">
          <h3>Activity log</h3>
          <span className="section-meta">Recent notes</span>
        </div>
        <ul className="console-log">
          {activityLog.map((entry) => (
            <li key={entry}>{entry}</li>
          ))}
        </ul>
      </section>
    </div>
  );
}
