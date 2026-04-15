import { useEffect } from "react";
import { HomePage } from "./routes/HomePage";
import { SettingsPage } from "./routes/SettingsPage";
import { WorkspacePage } from "./routes/WorkspacePage";
import { useUiStore } from "../stores/ui-store";
import { useWorkspaceStore } from "../stores/workspace-store";
import type { AppScreen } from "../types/models";

const navItems: AppScreen[] = ["home", "workspace", "settings"];

function screenLabel(screen: AppScreen) {
  if (screen === "home") {
    return "Home";
  }

  if (screen === "workspace") {
    return "Workspace";
  }

  return "Settings";
}

export default function App() {
  const currentScreen = useUiStore((state) => state.currentScreen);
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
  const status = useWorkspaceStore((state) => state.status);
  const errorMessage = useWorkspaceStore((state) => state.errorMessage);
  const loadBootstrap = useWorkspaceStore((state) => state.loadBootstrap);

  useEffect(() => {
    void loadBootstrap();
  }, [loadBootstrap]);

  return (
    <div className="app-shell">
      <header className="app-topbar">
        <div>
          <p className="eyebrow">Skill Notebook</p>
          <h1>Local skill workbench</h1>
        </div>
        <nav className="nav-row">
          {navItems.map((item) => (
            <button
              key={item}
              className={`nav-pill ${currentScreen === item ? "is-active" : ""}`}
              onClick={() => {
                setCurrentScreen(item);
              }}
              type="button"
            >
              {screenLabel(item)}
            </button>
          ))}
        </nav>
        <div className="topbar-status">
          <span className="status-led" />
          <span>{status}</span>
        </div>
      </header>

      {errorMessage ? <div className="error-banner">{errorMessage}</div> : null}

      <div className="window-stage">
        {currentScreen === "home" ? <HomePage /> : null}
        {currentScreen === "workspace" ? <WorkspacePage /> : null}
        {currentScreen === "settings" ? <SettingsPage /> : null}
      </div>
    </div>
  );
}
