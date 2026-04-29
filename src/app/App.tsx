import { useEffect } from "react";
import { CommandPalette } from "../components/command/CommandPalette";
import { WorkbenchView } from "./views/WorkbenchView";
import { CreateView } from "./views/CreateView";
import { SettingsPage } from "./routes/SettingsPage";
import { useUiStore } from "../stores/ui-store";
import { useProjectStore } from "../stores/project-store";

export default function App() {
  const currentScreen = useUiStore((state) => state.currentScreen);
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
  const openCommandPalette = useUiStore((state) => state.openCommandPalette);
  const status = useProjectStore((state) => state.status);
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const loadBootstrap = useProjectStore((state) => state.loadBootstrap);

  useEffect(() => {
    void loadBootstrap();
  }, [loadBootstrap]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        openCommandPalette();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [openCommandPalette]);

  const isSettings = currentScreen === "settings";
  const isCreate = currentScreen === "create";
  const projectRootPath = bootstrap?.projectRoot.rootPath ?? "项目根目录加载中";

  return (
    <div className="app-shell">
      <header className="app-topbar">
        <div className="topbar-left">
          <button
            className="topbar-brand"
            onClick={() => setCurrentScreen("explorer")}
            type="button"
          >
            Skill Notebook
          </button>
          <div className="topbar-project" title={projectRootPath}>
            <span>Project Root</span>
            <strong>{projectRootPath}</strong>
          </div>
        </div>
        <div className="topbar-right">
          <button
            className="topbar-command"
            onClick={openCommandPalette}
            type="button"
          >
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.35-4.35" />
            </svg>
            <span>搜索或命令</span>
            <kbd>⌘K</kbd>
          </button>
          <button
            className={`button-primary topbar-create ${isCreate ? "is-active" : ""}`}
            onClick={() => setCurrentScreen("create")}
            type="button"
          >
            生成 Skill
          </button>
          <span className={`status-led status-led-${status}`} />
          <button
            className={`topbar-gear ${isSettings ? "is-active" : ""}`}
            onClick={() => setCurrentScreen(isSettings ? "explorer" : "settings")}
            type="button"
            aria-label="设置"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
          </button>
        </div>
      </header>

      <div className="window-stage">
        {(currentScreen === "explorer" || currentScreen === "notebook") && <WorkbenchView />}
        {currentScreen === "create" && <CreateView />}
        {currentScreen === "settings" && <SettingsPage />}
      </div>
      <CommandPalette />
    </div>
  );
}
