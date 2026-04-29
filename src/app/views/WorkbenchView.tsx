import { useEffect, useState } from "react";
import { SkillLibraryColumn } from "../../components/library/SkillLibraryColumn";
import { FileColumnBrowser } from "../../components/browser/FileColumnBrowser";
import { EditorArea } from "../../components/notebook/EditorArea";
import { PackageMetadataPanel } from "../../components/notebook/PackageMetadataPanel";
import { useEditorStore } from "../../stores/editor-store";
import { useProjectStore } from "../../stores/project-store";
import { useUiStore } from "../../stores/ui-store";
import type { EvalReport, SkillPackage } from "../../types/models";

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

function EmptyContentPane({ evalReport, pkg }: { evalReport?: EvalReport; pkg: SkillPackage }) {
  return (
    <div className="workbench-empty-pane is-metadata">
      <PackageMetadataPanel evalReport={evalReport} pkg={pkg} />
      <div className="metadata-file-hint">
        <ArchiveIcon />
        <span>从左侧选择一个文件进入预览或编辑。</span>
        <ValidationPanel evalReport={evalReport} />
      </div>
    </div>
  );
}

type PendingNavigation =
  | { kind: "package"; packageId: string }
  | { kind: "file"; path: string };

function DirtySwitchModal({
  isSaving,
  onCancel,
  onDiscard,
  onSave,
}: {
  isSaving: boolean;
  onCancel: () => void;
  onDiscard: () => void;
  onSave: () => void;
}) {
  return (
    <div className="version-modal-overlay" onClick={onCancel} role="presentation">
      <div
        aria-labelledby="dirty-switch-title"
        aria-modal="true"
        className="version-modal version-action-modal dirty-switch-modal"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
      >
        <div className="version-modal-header">
          <div>
            <span className="version-modal-eyebrow is-danger">Unsaved</span>
            <h3 id="dirty-switch-title">当前文件有未保存修改</h3>
            <p className="muted version-modal-subtitle">切换前需要决定如何处理这份草稿。</p>
          </div>
          <button className="button-secondary version-modal-close" onClick={onCancel} type="button">
            关闭
          </button>
        </div>
        <div className="version-modal-body version-action-body">
          <div className="version-restore-warning">
            继续切换会替换当前编辑区。可以先保存，再继续切换；也可以放弃这次未保存修改。
          </div>
        </div>
        <div className="version-modal-footer">
          <button className="button-secondary" disabled={isSaving} onClick={onCancel} type="button">
            取消
          </button>
          <button className="button-secondary" disabled={isSaving} onClick={onDiscard} type="button">
            放弃修改
          </button>
          <button className="button-primary" disabled={isSaving} onClick={onSave} type="button">
            {isSaving ? "保存中..." : "保存并继续"}
          </button>
        </div>
      </div>
    </div>
  );
}

export function WorkbenchView() {
  const [pendingNavigation, setPendingNavigation] = useState<PendingNavigation | null>(null);
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const status = useProjectStore((state) => state.status);
  const selectedPackageId = useProjectStore((state) => state.selectedPackageId);
  const selectPackage = useProjectStore((state) => state.selectPackage);
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);

  const fileTree = useEditorStore((state) => state.fileTree);
  const currentFilePath = useEditorStore((state) => state.currentFilePath);
  const isTreeLoading = useEditorStore((state) => state.isTreeLoading);
  const isDirty = useEditorStore((state) => state.isDirty);
  const isSaving = useEditorStore((state) => state.isSaving);
  const fileError = useEditorStore((state) => state.fileError);
  const treeError = useEditorStore((state) => state.treeError);
  const loadFileTree = useEditorStore((state) => state.loadFileTree);
  const openFile = useEditorStore((state) => state.openFile);
  const saveFile = useEditorStore((state) => state.saveFile);
  const resetEditor = useEditorStore((state) => state.reset);

  const pkg = bootstrap?.packages.find((item) => item.id === selectedPackageId) ?? null;
  const evalReport = bootstrap?.evalReports.find((report) => report.packageId === selectedPackageId);

  useEffect(() => {
    if (!selectedPackageId) return;
    resetEditor();
    void loadFileTree(selectedPackageId);
  }, [loadFileTree, resetEditor, selectedPackageId]);

  useEffect(() => {
    if (!isDirty) return undefined;

    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };

    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [isDirty]);

  const runPendingNavigation = (pending: PendingNavigation) => {
    if (pending.kind === "package") {
      selectPackage(pending.packageId);
      setCurrentScreen("notebook");
      return;
    }

    if (pkg) {
      void openFile(pkg.id, pending.path);
    }
  };

  const requestPackageSelect = (packageId: string) => {
    if (isDirty && packageId !== selectedPackageId) {
      setPendingNavigation({ kind: "package", packageId });
      return;
    }
    selectPackage(packageId);
    setCurrentScreen("notebook");
  };

  const requestFileSelect = (path: string) => {
    if (isDirty && currentFilePath && path !== currentFilePath) {
      setPendingNavigation({ kind: "file", path });
      return;
    }
    if (pkg) {
      void openFile(pkg.id, path);
    }
  };

  const saveAndContinue = async () => {
    if (!pendingNavigation || !selectedPackageId) return;
    const saved = await saveFile(selectedPackageId);
    if (!saved) return;
    const pending = pendingNavigation;
    setPendingNavigation(null);
    runPendingNavigation(pending);
  };

  const discardAndContinue = () => {
    if (!pendingNavigation) return;
    const pending = pendingNavigation;
    setPendingNavigation(null);
    runPendingNavigation(pending);
  };

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
        onSelectPackage={requestPackageSelect}
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
            onSelectFile={requestFileSelect}
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
            <EmptyContentPane evalReport={evalReport} pkg={pkg} />
          )
        ) : (
          <div className="workbench-empty-pane is-large">
            <strong>Skill Notebook</strong>
            <span>选择、阅读、编辑、评估和版本化本地 skill。</span>
          </div>
        )}
      </main>
      {pendingNavigation ? (
        <DirtySwitchModal
          isSaving={isSaving}
          onCancel={() => setPendingNavigation(null)}
          onDiscard={discardAndContinue}
          onSave={() => { void saveAndContinue(); }}
        />
      ) : null}
    </section>
  );
}
