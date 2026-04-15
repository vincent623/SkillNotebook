import type { PreviewModel } from "../../types/models";

interface PackagePreviewProps {
  preview: PreviewModel;
}

export function PackagePreview({ preview }: PackagePreviewProps) {
  return (
    <div className="panel-scroll">
      <section className="content-card">
        <div className="section-head">
          <h3>SKILL.md preview</h3>
          <span className="section-meta">{preview.hasSkillMd ? "Present" : "Missing"}</span>
        </div>
        <p className="body-copy">{preview.skillMdPreview}</p>
      </section>

      <section className="content-card">
        <div className="section-head">
          <h3>Example output</h3>
          <span className="section-meta">Draft surface</span>
        </div>
        <p className="body-copy">{preview.examplePreview}</p>
      </section>

      <section className="content-card">
        <div className="section-head">
          <h3>Export preview</h3>
          <span className="section-meta">Downstream view</span>
        </div>
        <p className="body-copy">{preview.finalPreview}</p>
      </section>
    </div>
  );
}
