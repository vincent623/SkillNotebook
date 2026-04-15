import { useEffect, useState } from "react";
import { getSettings, openWorkspace } from "../../services/tauri-api";
import { useWorkspaceStore } from "../../stores/workspace-store";
import type { AppSettings } from "../../types/models";

const stack = [
  "Tauri 2",
  "Rust core",
  "React + TypeScript shell",
  "SQLite for metadata",
  "Filesystem package source",
  "skill-create draft bridge with Claude CLI + template fallback",
  "macOS local shell assumptions",
];

const commands = [
  "app_bootstrap",
  "workspace_open",
  "package_create_from_nl",
  "package_run_eval",
  "package_save_version",
  "package_run_test",
];

export function SettingsPage() {
  const bootstrap = useWorkspaceStore((state) => state.bootstrap);
  const loadBootstrap = useWorkspaceStore((state) => state.loadBootstrap);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [workspacePath, setWorkspacePath] = useState("");
  const [workspaceError, setWorkspaceError] = useState<string | null>(null);
  const [workspaceSuccess, setWorkspaceSuccess] = useState<string | null>(null);
  const [isSwitchingWorkspace, setIsSwitchingWorkspace] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function loadSettings() {
      try {
        const next = await getSettings();
        if (cancelled) {
          return;
        }

        setSettings(next);
        setWorkspacePath(next.currentWorkspaceRoot || (bootstrap?.workspace.rootPath ?? ""));
      } catch (error) {
        if (cancelled) {
          return;
        }

        setWorkspaceError(error instanceof Error ? error.message : "Failed to load settings.");
      }
    }

    void loadSettings();

    return () => {
      cancelled = true;
    };
  }, [bootstrap?.workspace.rootPath]);

  return (
    <section className="settings-page">
      <div className="settings-grid">
        <article className="content-card">
          <div className="section-head">
            <h3>Workspace control</h3>
            <span className="section-meta">Runtime context</span>
          </div>
          <label className="field-stack">
            <span className="search-label">Current workspace root</span>
            <input
              className="search-input"
              onChange={(event) => {
                setWorkspacePath(event.target.value);
              }}
              placeholder="/absolute/path/to/workspace"
              value={workspacePath}
            />
          </label>
          <div className="button-row">
            <button
              className="button-primary"
              disabled={isSwitchingWorkspace || !workspacePath.trim()}
              onClick={async () => {
                setIsSwitchingWorkspace(true);
                setWorkspaceError(null);
                setWorkspaceSuccess(null);

                try {
                  const workspace = await openWorkspace(workspacePath.trim());
                  await loadBootstrap();
                  const nextSettings = await getSettings();
                  setSettings(nextSettings);
                  setWorkspacePath(workspace.rootPath);
                  setWorkspaceSuccess(`Workspace switched to ${workspace.name}.`);
                } catch (error) {
                  setWorkspaceError(
                    error instanceof Error ? error.message : "Workspace switching failed.",
                  );
                } finally {
                  setIsSwitchingWorkspace(false);
                }
              }}
              type="button"
            >
              {isSwitchingWorkspace ? "Opening..." : "Open Workspace Path"}
            </button>
          </div>
          {workspaceSuccess ? (
            <div className="inline-banner inline-banner-success">{workspaceSuccess}</div>
          ) : null}
          {workspaceError ? (
            <div className="inline-banner inline-banner-error">{workspaceError}</div>
          ) : null}
          {settings?.recentWorkspaces?.length ? (
            <ul className="detail-list compact-list">
              {settings.recentWorkspaces.map((item) => (
                <li key={item.rootPath}>
                  <strong>{item.name}</strong>: <span className="mono-text">{item.rootPath}</span>
                </li>
              ))}
            </ul>
          ) : null}
        </article>

        <article className="content-card">
          <div className="section-head">
            <h3>Workspace target</h3>
            <span className="section-meta">Local first</span>
          </div>
          <p className="mono-text">{bootstrap?.workspace.rootPath ?? "No workspace path yet"}</p>
          <p className="muted">
            V1 assumes macOS, Apple Silicon first, and a local `zsh` or `bash` environment with
            optional `skill-create` (preferred) or Claude CLI draft bridges for richer first drafts.
          </p>
          <p className="muted">
            The active workspace is now a runtime context instead of a hardcoded default sample
            path, so bootstrap, create, and eval flows follow the last opened workspace.
          </p>
        </article>

        <article className="content-card">
          <div className="section-head">
            <h3>Stack</h3>
            <span className="section-meta">Confirmed direction</span>
          </div>
          <ul className="detail-list compact-list">
            {stack.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>

        <article className="content-card">
          <div className="section-head">
            <h3>Command surface</h3>
            <span className="section-meta">Rust surface</span>
          </div>
          <ul className="mono-list">
            {commands.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>
      </div>
    </section>
  );
}
