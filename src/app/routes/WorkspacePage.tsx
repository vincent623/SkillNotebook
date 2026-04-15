import { Sidebar } from "../../components/layout/Sidebar";
import { MainPanel } from "../../components/layout/MainPanel";
import { InspectorPanel } from "../../components/layout/InspectorPanel";
import { useWorkspaceStore } from "../../stores/workspace-store";

export function WorkspacePage() {
  const bootstrap = useWorkspaceStore((state) => state.bootstrap);
  const selectedPackageId = useWorkspaceStore((state) => state.selectedPackageId);
  const selectPackage = useWorkspaceStore((state) => state.selectPackage);

  if (!bootstrap) {
    return (
      <section className="loading-state">
        <p className="eyebrow">Workspace</p>
        <h2>Loading local bootstrap…</h2>
      </section>
    );
  }

  const selectedPackage = bootstrap.packages.find((item) => item.id === selectedPackageId);
  const evalReport = bootstrap.evalReports.find((item) => item.packageId === selectedPackageId);
  const preview = bootstrap.previews.find((item) => item.packageId === selectedPackageId);
  const versions = bootstrap.versions
    .filter((item) => item.packageId === selectedPackageId)
    .sort((left, right) => right.versionNumber - left.versionNumber);

  return (
    <section className="workspace-grid">
      <Sidebar
        workspace={bootstrap.workspace}
        packages={bootstrap.packages}
        selectedPackageId={selectedPackageId}
        onSelectPackage={selectPackage}
      />
      <MainPanel
        selectedPackage={selectedPackage}
        evalReport={evalReport}
        preview={preview}
        activityLog={bootstrap.activityLog}
      />
      <InspectorPanel
        selectedPackage={selectedPackage}
        evalReport={evalReport}
        preview={preview}
        versions={versions}
      />
    </section>
  );
}
