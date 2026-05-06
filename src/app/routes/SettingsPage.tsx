import { useEffect, useState } from "react";
import { createProjectRoot, getSettings, openProjectRoot } from "../../services/tauri-api";
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
  const [newProjectRootName, setNewProjectRootName] = useState("");
  const [isCreating, setIsCreating] = useState(false);

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

  async function switchProjectRoot(path: string) {
    setIsSwitching(true);
    setWorkspaceError(null);
    setWorkspaceSuccess(null);
    try {
      const projectRoot = await openProjectRoot(path.trim());
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
  }

  async function createAndOpenProjectRoot() {
    const name = newProjectRootName.trim();
    if (!name) return;
    setIsCreating(true);
    setWorkspaceError(null);
    setWorkspaceSuccess(null);
    try {
      const projectRoot = await createProjectRoot(name);
      await loadBootstrap();
      const next = await getSettings();
      setSettings(next);
      setWorkspacePath(projectRoot.rootPath);
      setNewProjectRootName("");
      setWorkspaceSuccess(`已创建并打开 ${projectRoot.rootPath}`);
    } catch (error) {
      setWorkspaceError(error instanceof Error ? error.message : "创建失败");
    } finally {
      setIsCreating(false);
    }
  }

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
              onClick={() => { void switchProjectRoot(projectRootPath); }}
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
                    <button
                      className="settings-recent-root"
                      disabled={isSwitching || item.rootPath === settings.currentProjectRoot}
                      onClick={() => { void switchProjectRoot(item.rootPath); }}
                      type="button"
                    >
                      <strong>{item.name}</strong>
                      <span className="mono-text">{item.rootPath}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </div>

        <div className="content-card">
          <h3>创建项目根目录</h3>
          <label className="field-stack" style={{ marginTop: 12 }}>
            <span className="field-label">名称</span>
            <input
              className="detail-save-input"
              onChange={(e) => setNewProjectRootName(e.target.value)}
              placeholder="例如 Research Skills"
              value={newProjectRootName}
            />
          </label>
          <p className="muted" style={{ marginTop: 8 }}>
            会在默认 project-root 同级目录创建新目录，并初始化 <span className="mono-text">.skills/</span> 与 <span className="mono-text">.skill-notebook/</span>。
          </p>
          <div style={{ marginTop: 8 }}>
            <button
              className="button-secondary"
              disabled={isCreating || !newProjectRootName.trim()}
              onClick={() => { void createAndOpenProjectRoot(); }}
              type="button"
            >
              {isCreating ? "创建中..." : "创建并打开"}
            </button>
          </div>
        </div>

        <div className="content-card">
          <h3>创建桥接</h3>
          {settings ? (
            <dl className="settings-bridge-grid" style={{ marginTop: 12 }}>
              <div>
                <dt>模式</dt>
                <dd>{settings.creationBridge.mode}</dd>
              </div>
              <div>
                <dt>优先生成器</dt>
                <dd>{settings.creationBridge.preferredGenerator}</dd>
              </div>
              <div>
                <dt>skill-create</dt>
                <dd>{settings.creationBridge.skillCreateCommandAvailable ? "可用" : "不可用"}</dd>
              </div>
              <div>
                <dt>Claude CLI</dt>
                <dd>
                  {settings.creationBridge.claudeCliAvailable ? "可用" : "不可用"}
                  {settings.creationBridge.claudeResolvedPath ? (
                    <code className="settings-bridge-path">{settings.creationBridge.claudeResolvedPath}</code>
                  ) : null}
                </dd>
              </div>
              <div>
                <dt>skill-create 路径</dt>
                <dd>
                  {settings.creationBridge.skillCreateResolvedPath ? (
                    <code className="settings-bridge-path">{settings.creationBridge.skillCreateResolvedPath}</code>
                  ) : (
                    "未解析"
                  )}
                </dd>
              </div>
              <div>
                <dt>Claude 超时</dt>
                <dd>{settings.creationBridge.claudeTimeoutSecs}s</dd>
              </div>
              <div>
                <dt>技能目录</dt>
                <dd>{settings.skillRootName ?? ".skills"}</dd>
              </div>
              <div>
                <dt>正式版本上限</dt>
                <dd>{settings.formalVersionCap}</dd>
              </div>
            </dl>
          ) : (
            <p className="muted" style={{ marginTop: 8 }}>正在读取创建桥接状态...</p>
          )}
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
