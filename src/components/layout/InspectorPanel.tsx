import { EvalDetails } from "../eval/EvalDetails";
import { EvalSummary } from "../eval/EvalSummary";
import { VersionDiffModal } from "../version/VersionDiffModal";
import { VersionList } from "../version/VersionList";
import { EmptyState } from "../common/EmptyState";
import { useWorkspaceStore } from "../../stores/workspace-store";
import type { EvalReport, PackageVersion, PreviewModel, SkillPackage } from "../../types/models";

interface InspectorPanelProps {
  selectedPackage?: SkillPackage;
  evalReport?: EvalReport;
  versions: PackageVersion[];
  preview?: PreviewModel;
}

export function InspectorPanel({
  selectedPackage,
  evalReport,
  versions,
  preview,
}: InspectorPanelProps) {
  const evalStatus = useWorkspaceStore((state) => state.evalStatus);
  const evalError = useWorkspaceStore((state) => state.evalError);
  const lastEvalPackageId = useWorkspaceStore((state) => state.lastEvalPackageId);
  const lastEvalCreatedAt = useWorkspaceStore((state) => state.lastEvalCreatedAt);
  const runEval = useWorkspaceStore((state) => state.runEval);

  if (!selectedPackage) {
    return (
      <aside className="workspace-inspector">
        <EmptyState
          title="Inspector is waiting"
          description="Metadata, eval, and version history appear here after a package is selected."
        />
      </aside>
    );
  }

  return (
    <aside className="workspace-inspector">
      <div className="content-card">
        <div className="section-head">
          <h3>Metadata</h3>
          <span className="section-meta">Package</span>
        </div>
        <dl className="meta-stack">
          <div>
            <dt>Slug</dt>
            <dd>{selectedPackage.slug}</dd>
          </div>
          <div>
            <dt>Path</dt>
            <dd className="mono-text">{selectedPackage.rootPath}</dd>
          </div>
          <div>
            <dt>Tags</dt>
            <dd>{selectedPackage.tags.join(", ")}</dd>
          </div>
          <div>
            <dt>Preview files</dt>
            <dd>{preview ? preview.promptFiles.length + preview.exampleFiles.length : 0}</dd>
          </div>
        </dl>
      </div>

      <EvalSummary
        errorMessage={lastEvalPackageId === selectedPackage.id ? evalError : null}
        isRunning={evalStatus === "submitting" && lastEvalPackageId === selectedPackage.id}
        lastRanAt={lastEvalPackageId === selectedPackage.id ? lastEvalCreatedAt : null}
        onRunEval={runEval}
        packageId={selectedPackage.id}
        report={evalReport}
      />
      <EvalDetails report={evalReport} />
      <VersionList
        packageId={selectedPackage.id}
        versions={versions}
        evalReport={evalReport}
      />
      <VersionDiffModal versions={versions} />
    </aside>
  );
}
