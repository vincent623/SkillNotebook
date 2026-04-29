import { useEffect, useState } from "react";
import { ExportUseModal } from "../../components/export/ExportUseModal";
import { SkillLibraryColumn } from "../../components/library/SkillLibraryColumn";
import { FileColumnBrowser } from "../../components/browser/FileColumnBrowser";
import { VersionPanel } from "../../components/notebook/VersionPanel";
import { EditorArea } from "../../components/notebook/EditorArea";
import { PackageSummary } from "../../components/notebook/PackageSummary";
import { useEditorStore } from "../../stores/editor-store";
import { useProjectStore } from "../../stores/project-store";
import { useUiStore } from "../../stores/ui-store";

export function WorkbenchView() {
  const [exportOpen, setExportOpen] = useState(false);
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const status = useProjectStore((state) => state.status);
  const selectedPackageId = useProjectStore((state) => state.selectedPackageId);
  const selectPackage = useProjectStore((state) => state.selectPackage);
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);

  const fileTree = useEditorStore((state) => state.fileTree);
  const currentFilePath = useEditorStore((state) => state.currentFilePath);
  const isTreeLoading = useEditorStore((state) => state.isTreeLoading);
  const fileError = useEditorStore((state) => state.fileError);
  const treeError = useEditorStore((state) => state.treeError);
  const loadFileTree = useEditorStore((state) => state.loadFileTree);
  const openFile = useEditorStore((state) => state.openFile);
  const resetEditor = useEditorStore((state) => state.reset);

  const pkg = bootstrap?.packages.find((item) => item.id === selectedPackageId) ?? null;
  const evalReport = bootstrap?.evalReports.find((report) => report.packageId === selectedPackageId);
  const versions = (bootstrap?.versions ?? [])
    .filter((version) => version.packageId === selectedPackageId)
    .sort((a, b) => b.versionNumber - a.versionNumber);

  useEffect(() => {
    if (!selectedPackageId) return;
    resetEditor();
    void loadFileTree(selectedPackageId);
  }, [loadFileTree, resetEditor, selectedPackageId]);

  if (!bootstrap) {
    return (
      <section className="workbench-loading">
        <span className={`status-led status-led-${status}`} />
        <p>正在加载技能仓库...</p>
      </section>
    );
  }

  return (
    <section className="workbench-view">
      <SkillLibraryColumn
        bootstrap={bootstrap}
        onCreate={() => setCurrentScreen("create")}
        onSelectPackage={(packageId) => {
          selectPackage(packageId);
          setCurrentScreen("notebook");
        }}
        selectedPackageId={selectedPackageId}
      />

      <aside className="workbench-browser-column" aria-label="Package Browser">
        {pkg ? (
          <>
            <div className="browser-package-header">
              <span className="browser-package-eyebrow">.skills/{pkg.slug}</span>
              <h2>{pkg.name}</h2>
              <p>{pkg.description}</p>
              <div className="browser-package-actions">
                <button
                  className="button-secondary browser-use-btn"
                  onClick={() => setExportOpen(true)}
                  type="button"
                >
                  使用 / 导出
                </button>
              </div>
            </div>
            <div className="browser-section">
              <div className="browser-section-label">文件</div>
              <FileColumnBrowser
                entries={fileTree}
                currentFilePath={currentFilePath}
                errorMessage={treeError}
                isLoading={isTreeLoading}
                packageSlug={pkg.slug}
                onSelectFile={(path) => { void openFile(pkg.id, path); }}
              />
            </div>
            <div className="browser-version-panel">
              <VersionPanel pkg={pkg} evalReport={evalReport} versions={versions} />
            </div>
          </>
        ) : (
          <div className="workbench-empty-pane">
            <strong>选择一个 Skill</strong>
            <span>左侧列表会显示当前项目根目录下的 `.skills/` 包。</span>
          </div>
        )}
      </aside>

      <main className="workbench-content-pane">
        {pkg ? (
          currentFilePath || fileError ? (
            <EditorArea packageId={pkg.id} />
          ) : (
            <PackageSummary pkg={pkg} evalReport={evalReport} />
          )
        ) : (
          <div className="workbench-empty-pane is-large">
            <strong>Skill Notebook</strong>
            <span>选择、阅读、编辑、评估和版本化本地 skill。</span>
          </div>
        )}
      </main>
      {exportOpen && pkg ? (
        <ExportUseModal
          onClose={() => setExportOpen(false)}
          pkg={pkg}
          projectRoot={bootstrap.projectRoot}
        />
      ) : null}
    </section>
  );
}
