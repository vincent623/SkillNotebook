import { useEffect, useMemo, useState } from "react";
import { CommandPalette } from "../components/command/CommandPalette";
import { ExportUseModal } from "../components/export/ExportUseModal";
import { WorkbenchView } from "./views/WorkbenchView";
import { CreateView } from "./views/CreateView";
import { SettingsPage } from "./routes/SettingsPage";
import { useUiStore } from "../stores/ui-store";
import { useProjectStore } from "../stores/project-store";
import type { EvalReport, SkillPackage } from "../types/models";

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

export default function App() {
  const [exportOpen, setExportOpen] = useState(false);
  const currentScreen = useUiStore((state) => state.currentScreen);
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
  const openCommandPalette = useUiStore((state) => state.openCommandPalette);
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const selectedPackageId = useProjectStore((state) => state.selectedPackageId);
  const loadBootstrap = useProjectStore((state) => state.loadBootstrap);
  const saveVersion = useProjectStore((state) => state.saveVersion);

  useEffect(() => {
    void loadBootstrap();
  }, [loadBootstrap]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        openCommandPalette();
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "n") {
        event.preventDefault();
        setCurrentScreen("create");
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [openCommandPalette, setCurrentScreen]);

  const selectedPackage = useMemo(
    () => bootstrap?.packages.find((item) => item.id === selectedPackageId) ?? bootstrap?.packages[0] ?? null,
    [bootstrap?.packages, selectedPackageId],
  );
  const selectedEvalReport = useMemo(
    () => bootstrap?.evalReports.find((report) => report.packageId === selectedPackage?.id),
    [bootstrap?.evalReports, selectedPackage?.id],
  );
  const quality = selectedQuality(selectedPackage, selectedEvalReport);

  function handleSaveVersion() {
    if (!selectedPackage) return;
    const note = window.prompt(`保存 ${selectedPackage.slug} 的新版本`, "更新");
    if (note === null) return;
    void saveVersion(selectedPackage.id, note.trim() || "更新");
  }

  return (
    <div className="app-shell">
      <header className="app-topbar">
        <div className="topbar-left">
          <button
            className="topbar-brand"
            onClick={() => setCurrentScreen("explorer")}
            type="button"
          >
            <span className="topbar-brand-mark">技</span>
            <span>技能本</span>
          </button>
          {selectedPackage ? (
            <div className="topbar-breadcrumb" title={selectedPackage.rootPath}>
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
            className="topbar-create"
            onClick={() => setCurrentScreen("create")}
            type="button"
          >
            <WandIcon />
            生成 Skill
          </button>
          <button
            className="topbar-icon-button"
            disabled={!selectedPackage}
            onClick={handleSaveVersion}
            title="提交新版本"
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
            导出
          </button>
          <button className="topbar-icon-button" disabled title="删除" type="button">
            <TrashIcon />
          </button>
        </div>
      </header>

      <div className="window-stage">
        {(currentScreen === "explorer" || currentScreen === "notebook") && <WorkbenchView />}
        {currentScreen === "create" && <CreateView />}
        {currentScreen === "settings" && <SettingsPage />}
      </div>
      <CommandPalette />
      {exportOpen && bootstrap && selectedPackage ? (
        <ExportUseModal
          onClose={() => setExportOpen(false)}
          pkg={selectedPackage}
          projectRoot={bootstrap.projectRoot}
        />
      ) : null}
    </div>
  );
}
