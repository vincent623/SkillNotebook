import { useCallback, useEffect, useMemo, useState } from "react";
import { CommandPalette } from "../components/command/CommandPalette";
import { QuickReferenceModal } from "../components/reference/QuickReferenceModal";
import { VersionPanel } from "../components/notebook/VersionPanel";
import { WorkbenchView } from "./views/WorkbenchView";
import { DraftImportView } from "./views/DraftImportView";
import { SettingsPage } from "./routes/SettingsPage";
import { useUiStore } from "../stores/ui-store";
import { useEditorStore } from "../stores/editor-store";
import { useProjectStore } from "../stores/project-store";
import type { AppScreen, EvalReport, SkillPackage } from "../types/models";

function SearchIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" />
    </svg>
  );
}

function WandIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 13l9-9M9.5 3.5 12.5 6.5" />
      <path d="m13 9 .5 1.5L15 11l-1.5.5L13 13l-.5-1.5L11 11l1.5-.5Z" />
    </svg>
  );
}

function GitIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="4" cy="4" r="1.5" />
      <circle cx="12" cy="8" r="1.5" />
      <circle cx="4" cy="12" r="1.5" />
      <path d="M5.5 4.5H9A1.5 1.5 0 0 1 10.5 6v.5M4 5.5v5" />
    </svg>
  );
}

function DownloadIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M8 2v8M4 7l4 4 4-4M3 13h10" />
    </svg>
  );
}

function GearIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="8" cy="8" r="2.3" />
      <path d="M8 1.8v2M8 12.2v2M3.6 3.6 5 5M11 11l1.4 1.4M1.8 8h2M12.2 8h2M3.6 12.4 5 11M11 5l1.4-1.4" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 4.5h10M6 4.5V3h4v1.5M4.5 4.5 5 13a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1l.5-8.5" />
    </svg>
  );
}

function WarningIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M8 2 14.5 13H1.5Z" />
      <path d="M8 6.5v3M8 11.5v.5" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 8 6.5 11.5 13 5" />
    </svg>
  );
}

function selectedQuality(pkg: SkillPackage | null, report?: EvalReport) {
  if (!pkg) return null;
  if (report && report.suggestions.length > 0) {
    return { tone: "warn", label: `${report.suggestions.length}` };
  }
  if (report?.overallStatus === "problematic") {
    return { tone: "error", label: `${report.suggestions.length || 1}` };
  }
  if (report?.overallStatus === "needs_improvement") {
    return { tone: "warn", label: `${report.suggestions.length || 1}` };
  }
  if (report?.overallStatus === "usable" || pkg.status === "validated") {
    return { tone: "ok", label: "校验通过" };
  }
  return { tone: "warn", label: "1" };
}

function DirtyNavigationModal({
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
        aria-labelledby="dirty-nav-title"
        aria-modal="true"
        className="version-modal version-action-modal dirty-switch-modal"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
      >
        <div className="version-modal-header">
          <div>
            <span className="version-modal-eyebrow is-danger">Unsaved</span>
            <h3 id="dirty-nav-title">当前文件还没保存</h3>
            <p className="muted version-modal-subtitle">离开工作台前，请先处理这份修改。</p>
          </div>
          <button className="button-secondary version-modal-close" onClick={onCancel} type="button">
            关闭
          </button>
        </div>
        <div className="version-modal-body version-action-body">
          <div className="version-restore-warning">离开工作台会重新装载当前文件。保存后继续，或放弃这次修改。</div>
        </div>
        <div className="version-modal-footer">
          <button className="button-secondary" disabled={isSaving} onClick={onCancel} type="button">取消</button>
          <button className="button-secondary" disabled={isSaving} onClick={onDiscard} type="button">放弃修改</button>
          <button className="button-primary" disabled={isSaving} onClick={onSave} type="button">
            {isSaving ? "保存中..." : "保存并继续"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default function App() {
  const [exportOpen, setExportOpen] = useState(false);
  const [pendingScreen, setPendingScreen] = useState<AppScreen | null>(null);
  const currentScreen = useUiStore((state) => state.currentScreen);
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
  const openCommandPalette = useUiStore((state) => state.openCommandPalette);
  const isVersionPanelOpen = useUiStore((state) => state.isVersionPanelOpen);
  const openVersionPanel = useUiStore((state) => state.openVersionPanel);
  const closeVersionPanel = useUiStore((state) => state.closeVersionPanel);
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const status = useProjectStore((state) => state.status);
  const selectedPackageId = useProjectStore((state) => state.selectedPackageId);
  const loadBootstrap = useProjectStore((state) => state.loadBootstrap);
  const refreshBootstrap = useProjectStore((state) => state.refreshBootstrap);
  const isDirty = useEditorStore((state) => state.isDirty);
  const isSaving = useEditorStore((state) => state.isSaving);
  const saveFile = useEditorStore((state) => state.saveFile);
  const loadFileTree = useEditorStore((state) => state.loadFileTree);
  const refreshOpenFile = useEditorStore((state) => state.refreshOpenFile);

  useEffect(() => {
    void loadBootstrap();
  }, [loadBootstrap]);

  useEffect(() => {
    const handleOpenReference = () => setExportOpen(true);
    window.addEventListener("skillnotebook:open-reference", handleOpenReference);
    return () => window.removeEventListener("skillnotebook:open-reference", handleOpenReference);
  }, []);

  useEffect(() => {
    if (currentScreen !== "explorer" && currentScreen !== "notebook") return undefined;

    const intervalId = window.setInterval(() => {
      if (document.visibilityState !== "visible" || isDirty) return;
      void refreshBootstrap(selectedPackageId);
      if (selectedPackageId) {
        void loadFileTree(selectedPackageId);
        void refreshOpenFile(selectedPackageId);
      }
    }, 5000);

    return () => window.clearInterval(intervalId);
  }, [currentScreen, isDirty, loadFileTree, refreshBootstrap, refreshOpenFile, selectedPackageId]);

  const requestScreen = useCallback((screen: AppScreen) => {
    if (
      isDirty &&
      selectedPackageId &&
      (currentScreen === "explorer" || currentScreen === "notebook") &&
      screen !== currentScreen
    ) {
      setPendingScreen(screen);
      return;
    }
    setCurrentScreen(screen);
  }, [currentScreen, isDirty, selectedPackageId, setCurrentScreen]);

  const continueToPendingScreen = useCallback(() => {
    if (!pendingScreen) return;
    setCurrentScreen(pendingScreen);
    setPendingScreen(null);
  }, [pendingScreen, setCurrentScreen]);

  const saveAndContinueToPendingScreen = useCallback(async () => {
    if (!pendingScreen || !selectedPackageId) return;
    const saved = await saveFile(selectedPackageId);
    if (!saved) return;
    continueToPendingScreen();
  }, [continueToPendingScreen, pendingScreen, saveFile, selectedPackageId]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        openCommandPalette();
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "n") {
        event.preventDefault();
        if (isDirty && selectedPackageId && (currentScreen === "explorer" || currentScreen === "notebook")) {
          setPendingScreen("draft");
        } else {
          setCurrentScreen("draft");
        }
      }
      if (event.key === "Escape") {
        closeVersionPanel();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [closeVersionPanel, currentScreen, isDirty, openCommandPalette, selectedPackageId, setCurrentScreen]);

  const selectedPackage = useMemo(
    () => bootstrap?.packages.find((item) => item.id === selectedPackageId) ?? bootstrap?.packages[0] ?? null,
    [bootstrap?.packages, selectedPackageId],
  );
  const selectedEvalReport = useMemo(
    () => bootstrap?.evalReports.find((report) => report.packageId === selectedPackage?.id),
    [bootstrap?.evalReports, selectedPackage?.id],
  );
  const selectedVersions = useMemo(
    () =>
      (bootstrap?.versions ?? [])
        .filter((version) => version.packageId === selectedPackage?.id)
        .sort((a, b) => b.versionNumber - a.versionNumber),
    [bootstrap?.versions, selectedPackage?.id],
  );
  const quality = selectedQuality(selectedPackage, selectedEvalReport);
  const projectRootPath = bootstrap?.projectRoot.rootPath ?? "尚未加载项目根目录";
  const statusLabel = status === "loading" ? "加载中" : status === "error" ? "错误" : status === "ready" ? "就绪" : "待机";

  return (
    <div className="app-shell">
      <header className="app-topbar">
        <div className="topbar-left" data-tauri-drag-region>
          <button
            className="topbar-brand"
            onClick={() => requestScreen("explorer")}
            aria-label="打开 Skill Notebook 工作台"
            type="button"
          >
            <span className="topbar-brand-mark" aria-hidden="true" />
            <span className="topbar-brand-copy">
              <strong>Skill Notebook</strong>
              <small>技能本</small>
            </span>
          </button>
          <div className="topbar-project" data-tauri-drag-region title={projectRootPath}>
            <span>
              <i className={`status-led status-led-${status}`} />
              {statusLabel}
            </span>
            <strong>{projectRootPath}</strong>
          </div>
          {selectedPackage ? (
            <div className="topbar-breadcrumb" data-tauri-drag-region title={selectedPackage.rootPath}>
              <span>›</span>
              <code>{selectedPackage.slug}/</code>
              <code className="topbar-version-pill">v{selectedPackage.currentVersion}</code>
              {quality ? (
                <span className={`topbar-validation topbar-validation-${quality.tone}`}>
                  {quality.tone === "ok" ? <CheckIcon /> : <WarningIcon />}
                  {quality.label}
                </span>
              ) : null}
            </div>
          ) : null}
        </div>
        <div className="topbar-right">
          <button
            className="topbar-command"
            onClick={openCommandPalette}
            type="button"
          >
            <SearchIcon />
            <span>搜索</span>
            <kbd>⌘K</kbd>
          </button>
          <button
            className={`topbar-draft ${currentScreen === "draft" ? "is-active" : ""}`}
            onClick={() => requestScreen("draft")}
            type="button"
          >
            <WandIcon />
            导入
          </button>
          <button
            className="topbar-icon-button"
            disabled={!selectedPackage}
            onClick={openVersionPanel}
            title="版本与质量门禁"
            type="button"
          >
            <GitIcon />
          </button>
          <button
            className="topbar-secondary"
            disabled={!selectedPackage}
            onClick={() => setExportOpen(true)}
            type="button"
          >
            <DownloadIcon />
            快速引用
          </button>
          <button
            className={`topbar-icon-button ${currentScreen === "settings" ? "is-active" : ""}`}
            onClick={() => requestScreen("settings")}
            title="设置"
            type="button"
          >
            <GearIcon />
          </button>
          <button className="topbar-icon-button" disabled title="删除" type="button">
            <TrashIcon />
          </button>
        </div>
      </header>

      <div className="window-stage">
        <WorkbenchView />
      </div>
      <CommandPalette />
      {currentScreen === "draft" && <DraftImportView />}
      {currentScreen === "settings" && <SettingsPage />}
      {exportOpen && bootstrap && selectedPackage ? (
        <QuickReferenceModal
          onClose={() => setExportOpen(false)}
          pkg={selectedPackage}
          projectRoot={bootstrap.projectRoot}
        />
      ) : null}
      {isVersionPanelOpen && selectedPackage ? (
        <div className="version-drawer-overlay" onClick={closeVersionPanel} role="presentation">
          <section
            aria-label="版本与质量门禁"
            className="version-drawer"
            onClick={(event) => event.stopPropagation()}
          >
            <header className="version-drawer-header">
              <div>
                <span className="version-drawer-eyebrow">Versions</span>
                <h2>版本与质量门禁</h2>
                <p>{selectedPackage.slug} · v{selectedPackage.currentVersion}</p>
              </div>
              <button className="button-secondary version-drawer-close" onClick={closeVersionPanel} type="button">
                关闭
              </button>
            </header>
            <div className="version-drawer-body">
              <VersionPanel
                evalReport={selectedEvalReport}
                pkg={selectedPackage}
                versions={selectedVersions}
              />
            </div>
          </section>
        </div>
      ) : null}
      {pendingScreen ? (
        <DirtyNavigationModal
          isSaving={isSaving}
          onCancel={() => setPendingScreen(null)}
          onDiscard={continueToPendingScreen}
          onSave={() => { void saveAndContinueToPendingScreen(); }}
        />
      ) : null}
    </div>
  );
}
