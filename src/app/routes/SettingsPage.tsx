import { useEffect, useState } from "react";
import { createProjectRoot, getSettings, openProjectRoot, updateSettings } from "../../services/tauri-api";
import { BackButton } from "../../components/common/BackButton";
import { useProjectStore } from "../../stores/project-store";
import type { AppSettings } from "../../types/models";

export function SettingsPage() {
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const loadBootstrap = useProjectStore((state) => state.loadBootstrap);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [projectRootPath, setProjectRootPath] = useState("");
  const [projectRootError, setProjectRootError] = useState<string | null>(null);
  const [projectRootSuccess, setProjectRootSuccess] = useState<string | null>(null);
  const [isSwitching, setIsSwitching] = useState(false);
  const [newProjectRootName, setNewProjectRootName] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const [terminalCommand, setTerminalCommand] = useState("open -a Terminal");
  const [editorCommand, setEditorCommand] = useState("");
  const [agentCommand, setAgentCommand] = useState("codex");
  const [globalClaudeSkillsDir, setGlobalClaudeSkillsDir] = useState("~/.claude/skills");
  const [projectClaudeSkillsDirName, setProjectClaudeSkillsDirName] = useState(".claude/skills");
  const [handoffError, setHandoffError] = useState<string | null>(null);
  const [handoffSuccess, setHandoffSuccess] = useState<string | null>(null);
  const [isSavingHandoff, setIsSavingHandoff] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const next = await getSettings();
        if (cancelled) return;
        setSettings(next);
        populateHandoffForm(next);
        setProjectRootPath(next.currentProjectRoot || (bootstrap?.projectRoot.rootPath ?? ""));
      } catch (error) {
        if (!cancelled) setProjectRootError(error instanceof Error ? error.message : "加载设置失败");
      }
    }
    void load();
    return () => { cancelled = true; };
  }, [bootstrap?.projectRoot.rootPath]);

  function populateHandoffForm(next: AppSettings) {
    setTerminalCommand(next.handoff.terminalCommand ?? "open -a Terminal");
    setEditorCommand(next.handoff.editorCommand ?? "");
    setAgentCommand(next.handoff.agentCommand ?? "codex");
    setGlobalClaudeSkillsDir(next.handoff.globalClaudeSkillsDir ?? "~/.claude/skills");
    setProjectClaudeSkillsDirName(next.handoff.projectClaudeSkillsDirName ?? ".claude/skills");
  }

  async function switchProjectRoot(path: string) {
    setIsSwitching(true);
    setProjectRootError(null);
    setProjectRootSuccess(null);
    try {
      const projectRoot = await openProjectRoot(path.trim());
      await loadBootstrap();
      const next = await getSettings();
      setSettings(next);
      populateHandoffForm(next);
      setProjectRootPath(projectRoot.rootPath);
      setProjectRootSuccess(`已切换到 ${projectRoot.rootPath}`);
    } catch (error) {
      setProjectRootError(error instanceof Error ? error.message : "切换失败");
    } finally {
      setIsSwitching(false);
    }
  }

  async function createAndOpenProjectRoot() {
    const name = newProjectRootName.trim();
    if (!name) return;
    setIsCreating(true);
    setProjectRootError(null);
    setProjectRootSuccess(null);
    try {
      const projectRoot = await createProjectRoot(name);
      await loadBootstrap();
      const next = await getSettings();
      setSettings(next);
      populateHandoffForm(next);
      setProjectRootPath(projectRoot.rootPath);
      setNewProjectRootName("");
      setProjectRootSuccess(`已创建并打开 ${projectRoot.rootPath}`);
    } catch (error) {
      setProjectRootError(error instanceof Error ? error.message : "创建失败");
    } finally {
      setIsCreating(false);
    }
  }

  async function saveHandoffConfig() {
    setIsSavingHandoff(true);
    setHandoffError(null);
    setHandoffSuccess(null);
    try {
      const next = await updateSettings({
        handoff: {
          terminalCommand: terminalCommand.trim() || null,
          editorCommand: editorCommand.trim() || null,
          agentCommand: agentCommand.trim() || null,
          globalClaudeSkillsDir: globalClaudeSkillsDir.trim() || null,
          projectClaudeSkillsDirName: projectClaudeSkillsDirName.trim() || null,
        },
      });
      setSettings(next);
      populateHandoffForm(next);
      setHandoffSuccess("本地交接偏好已保存。");
    } catch (error) {
      setHandoffError(error instanceof Error ? error.message : "保存交接偏好失败");
    } finally {
      setIsSavingHandoff(false);
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
              onChange={(e) => setProjectRootPath(e.target.value)}
              placeholder="/absolute/path/to/project-root"
              value={projectRootPath}
            />
          </label>
          <p className="muted" style={{ marginTop: 8 }}>
            Skill Notebook 会固定从这个根目录下的 <span className="mono-text">.skills/</span> 读取和管理所有 skill。
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
          <h3>本地交接偏好</h3>
          {settings ? (
            <>
              <div className="agent-config-grid" style={{ marginTop: 12 }}>
                <label className="field-stack">
                  <span className="field-label">Terminal command</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setTerminalCommand(e.target.value)}
                    placeholder="open -a Terminal"
                    value={terminalCommand}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">Editor command</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setEditorCommand(e.target.value)}
                    placeholder="code"
                    value={editorCommand}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">External agent command</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setAgentCommand(e.target.value)}
                    placeholder="codex"
                    value={agentCommand}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">Global Claude skills</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setGlobalClaudeSkillsDir(e.target.value)}
                    placeholder="~/.claude/skills"
                    value={globalClaudeSkillsDir}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">Project Claude skills</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setProjectClaudeSkillsDirName(e.target.value)}
                    placeholder=".claude/skills"
                    value={projectClaudeSkillsDirName}
                  />
                </label>
              </div>
              <p className="muted" style={{ marginTop: 8 }}>
                这些设置只影响草稿交接、快速引用和本地命令展示；Skill Notebook 不保存模型 API key。
              </p>
              <div className="settings-action-row">
                <button
                  className="button-primary"
                  disabled={isSavingHandoff}
                  onClick={() => { void saveHandoffConfig(); }}
                  type="button"
                >
                  {isSavingHandoff ? "保存中..." : "保存交接偏好"}
                </button>
              </div>
              {handoffSuccess ? <div className="inline-banner inline-banner-success">{handoffSuccess}</div> : null}
              {handoffError ? <div className="inline-banner inline-banner-error">{handoffError}</div> : null}
            </>
          ) : (
            <p className="muted" style={{ marginTop: 8 }}>正在读取本地交接偏好...</p>
          )}
        </div>

        <div className="content-card">
          <h3>本地状态</h3>
          {settings ? (
            <dl className="settings-bridge-grid" style={{ marginTop: 12 }}>
              <div>
                <dt>技能目录</dt>
                <dd>{settings.skillRootName ?? ".skills"}</dd>
              </div>
              <div>
                <dt>正式版本上限</dt>
                <dd>{settings.formalVersionCap}</dd>
              </div>
              <div>
                <dt>默认项目根目录</dt>
                <dd><code className="settings-bridge-path">{settings.defaultProjectRoot}</code></dd>
              </div>
              <div>
                <dt>设置文件</dt>
                <dd>{settings.settingsPath ? <code className="settings-bridge-path">{settings.settingsPath}</code> : "未解析"}</dd>
              </div>
              <div>
                <dt>Shell</dt>
                <dd>{settings.shell.join(" / ")}</dd>
              </div>
            </dl>
          ) : (
            <p className="muted" style={{ marginTop: 8 }}>正在读取本地状态...</p>
          )}
        </div>

        <div className="content-card">
          <h3>关于</h3>
          <p className="body-copy" style={{ marginTop: 8 }}>
            Skill Notebook v{import.meta.env.VITE_APP_VERSION} — macOS 本地优先的 skill 资产管理、评估、版本和快速引用工具。
          </p>
          <p className="muted" style={{ marginTop: 4 }}>
            Tauri 2 + Rust + React + TypeScript · Apple Silicon 适配
          </p>
        </div>
      </div>
    </section>
  );
}
