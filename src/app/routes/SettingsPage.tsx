import { useEffect, useState } from "react";
import { getSettings, openProjectRoot } from "../../services/tauri-api";
import { BackButton } from "../../components/common/BackButton";
import { useProjectStore } from "../../stores/project-store";
import type { AppSettings } from "../../types/models";

export function SettingsPage() {
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const loadBootstrap = useProjectStore((state) => state.loadBootstrap);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [projectRootPath, setWorkspacePath] = useState("");
  const [projectRootError, setWorkspaceError] = useState<string | null>(null);
  const [projectRootSuccess, setWorkspaceSuccess] = useState<string | null>(null);
  const [isSwitching, setIsSwitching] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const next = await getSettings();
        if (cancelled) return;
        setSettings(next);
        setWorkspacePath(next.currentProjectRoot || (bootstrap?.projectRoot.rootPath ?? ""));
      } catch (error) {
        if (!cancelled) setWorkspaceError(error instanceof Error ? error.message : "加载设置失败");
      }
    }
    void load();
    return () => { cancelled = true; };
  }, [bootstrap?.projectRoot.rootPath]);

  return (
    <section className="settings-view">
      <BackButton />
      <div className="settings-scroll">
        <div className="content-card">
          <h3>项目根目录</h3>
          <label className="field-stack" style={{ marginTop: 12 }}>
            <span className="field-label">仓库路径</span>
            <input
              className="detail-save-input"
              onChange={(e) => setWorkspacePath(e.target.value)}
              placeholder="/absolute/path/to/project-root"
              value={projectRootPath}
            />
          </label>
          <p className="muted" style={{ marginTop: 8 }}>
            Skill Notebook 会固定从这个根目录下的 <span className="mono-text">.skills/</span> 读取和创建所有 skill。
          </p>
          <div style={{ marginTop: 8 }}>
            <button
              className="button-primary"
              disabled={isSwitching || !projectRootPath.trim()}
              onClick={async () => {
                setIsSwitching(true);
                setWorkspaceError(null);
                setWorkspaceSuccess(null);
                try {
                  const projectRoot = await openProjectRoot(projectRootPath.trim());
                  await loadBootstrap();
                  const next = await getSettings();
                  setSettings(next);
                  setWorkspacePath(projectRoot.rootPath);
                  setWorkspaceSuccess(`已切换到 ${projectRoot.rootPath}`);
                } catch (error) {
                  setWorkspaceError(error instanceof Error ? error.message : "切换失败");
                } finally {
                  setIsSwitching(false);
                }
              }}
              type="button"
            >
              {isSwitching ? "切换中..." : "打开项目"}
            </button>
          </div>
          {projectRootSuccess ? <div className="inline-banner inline-banner-success">{projectRootSuccess}</div> : null}
          {projectRootError ? <div className="inline-banner inline-banner-error">{projectRootError}</div> : null}
          {settings?.recentProjectRoots?.length ? (
            <div style={{ marginTop: 12 }}>
              <span className="field-label">最近使用</span>
              <ul className="detail-suggestions" style={{ marginTop: 4 }}>
                {settings.recentProjectRoots.map((item) => (
                  <li key={item.rootPath}>
                    <strong>{item.name}</strong> <span className="mono-text">{item.rootPath}</span>
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </div>

        <div className="content-card">
          <h3>关于</h3>
          <p className="body-copy" style={{ marginTop: 8 }}>
            Skill Notebook v0.1.0 — macOS 本地优先的 skill 仓库与版本管理工具。
          </p>
          <p className="muted" style={{ marginTop: 4 }}>
            Tauri 2 + Rust + React + TypeScript · Apple Silicon 适配
          </p>
        </div>
      </div>
    </section>
  );
}
