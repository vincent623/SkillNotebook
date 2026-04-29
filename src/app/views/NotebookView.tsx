import { useEffect } from "react";
import { BackButton } from "../../components/common/BackButton";
import { FileTree } from "../../components/notebook/FileTree";
import { VersionPanel } from "../../components/notebook/VersionPanel";
import { EditorArea } from "../../components/notebook/EditorArea";
import { PackageSummary } from "../../components/notebook/PackageSummary";
import { useProjectStore } from "../../stores/project-store";
import { useEditorStore } from "../../stores/editor-store";

export function NotebookView() {
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const selectedPackageId = useProjectStore((state) => state.selectedPackageId);

  const fileTree = useEditorStore((state) => state.fileTree);
  const currentFilePath = useEditorStore((state) => state.currentFilePath);
  const isTreeLoading = useEditorStore((state) => state.isTreeLoading);
  const fileError = useEditorStore((state) => state.fileError);
  const treeError = useEditorStore((state) => state.treeError);
  const loadFileTree = useEditorStore((state) => state.loadFileTree);
  const openFile = useEditorStore((state) => state.openFile);
  const reset = useEditorStore((state) => state.reset);

  const pkg = bootstrap?.packages.find((p) => p.id === selectedPackageId);
  const evalReport = bootstrap?.evalReports.find((r) => r.packageId === selectedPackageId);
  const versions = (bootstrap?.versions ?? [])
    .filter((v) => v.packageId === selectedPackageId)
    .sort((a, b) => b.versionNumber - a.versionNumber);

  useEffect(() => {
    if (selectedPackageId) {
      reset();
      void loadFileTree(selectedPackageId);
    }
  }, [selectedPackageId, loadFileTree, reset]);

  if (!bootstrap || !pkg) return null;

  const handleSelectFile = (path: string) => {
    void openFile(pkg.id, path);
  };

  return (
    <section className="notebook-view">
      <div className="notebook-sidebar">
        <BackButton />
        <div className="notebook-sidebar-upper">
          <FileTree
            entries={fileTree}
            currentFilePath={currentFilePath}
            onSelectFile={handleSelectFile}
            isLoading={isTreeLoading}
            errorMessage={treeError}
          />
        </div>
        <div className="notebook-sidebar-lower">
          <VersionPanel pkg={pkg} evalReport={evalReport} versions={versions} />
        </div>
      </div>
      <div className="notebook-main">
        {currentFilePath || fileError ? (
          <EditorArea packageId={pkg.id} />
        ) : (
          <PackageSummary pkg={pkg} evalReport={evalReport} />
        )}
      </div>
    </section>
  );
}
