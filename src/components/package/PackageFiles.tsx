import type { PreviewModel } from "../../types/models";

interface PackageFilesProps {
  preview: PreviewModel;
}

function FileBlock({ label, items }: { label: string; items: string[] }) {
  return (
    <div className="file-block">
      <div className="section-head">
        <h3>{label}</h3>
        <span className="section-meta">{items.length}</span>
      </div>
      <ul className="mono-list">
        {items.length === 0 ? <li className="muted">None yet</li> : null}
        {items.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </div>
  );
}

export function PackageFiles({ preview }: PackageFilesProps) {
  return (
    <div className="panel-scroll">
      <section className="content-card">
        <div className="section-head">
          <h3>Package map</h3>
          <span className="section-meta">{preview.name}</span>
        </div>
        <div className="file-grid">
          <FileBlock
            label="Prompts"
            items={preview.promptFiles}
          />
          <FileBlock
            label="Examples"
            items={preview.exampleFiles}
          />
          <FileBlock
            label="References"
            items={preview.referenceFiles}
          />
          <FileBlock
            label="Scripts"
            items={preview.scriptFiles}
          />
          <FileBlock
            label="Tests"
            items={preview.testFiles}
          />
        </div>
      </section>
    </div>
  );
}
