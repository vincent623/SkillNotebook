import { useEffect } from "react";
import { SkillLibraryColumn } from "../../components/library/SkillLibraryColumn";
import { FileColumnBrowser } from "../../components/browser/FileColumnBrowser";
import { EditorArea } from "../../components/notebook/EditorArea";
import { useEditorStore } from "../../stores/editor-store";
import { useProjectStore } from "../../stores/project-store";
import { useUiStore } from "../../stores/ui-store";
import type { EvalReport } from "../../types/models";

function ArchiveIcon() {
  return (
    <svg width="32" height="32" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="3" width="12" height="3" rx=".5" />
      <path d="M3 6v7a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V6M6 9h4" />
    </svg>
  );
}

function ValidationPanel({ evalReport }: { evalReport?: EvalReport }) {
  if (!evalReport) return null;

  if (evalReport.overallStatus === "usable" && evalReport.suggestions.length === 0) {
    return (
      <div className="prototype-validation-panel prototype-validation-ok">
        <span>✓</span>
        全部校验通过
      </div>
    );
  }

  return (
    <div className="prototype-validation-panel">
      <div className="prototype-validation-title">校验结果</div>
      {(evalReport.suggestions.length > 0 ? evalReport.suggestions : ["建议运行评估并检查 skill 结构。"])
        .slice(0, 3)
        .map((suggestion) => (
          <div className="prototype-validation-row" key={suggestion}>
            <span>△</span>
            {suggestion}
          </div>
        ))}
    </div>
  );
}

function EmptyContentPane({ evalReport }: { evalReport?: EvalReport }) {
  return (
    <div className="workbench-empty-pane is-large">
      <ArchiveIcon />
      <span>从左侧选择一个文件</span>
      <ValidationPanel evalReport={evalReport} />
    </div>
  );
}

export function WorkbenchView() {
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
          <FileColumnBrowser
            entries={fileTree}
            currentFilePath={currentFilePath}
            errorMessage={treeError}
            isLoading={isTreeLoading}
            packageSlug={pkg.slug}
            onSelectFile={(path) => { void openFile(pkg.id, path); }}
          />
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
            <EmptyContentPane evalReport={evalReport} />
          )
        ) : (
          <div className="workbench-empty-pane is-large">
            <strong>Skill Notebook</strong>
            <span>选择、阅读、编辑、评估和版本化本地 skill。</span>
          </div>
        )}
      </main>
    </section>
  );
}
