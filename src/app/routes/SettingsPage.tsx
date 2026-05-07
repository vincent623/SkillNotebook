import { useEffect, useState } from "react";
import { createProjectRoot, getSettings, openProjectRoot, updateSettings } from "../../services/tauri-api";
import { useProjectStore } from "../../stores/project-store";
import { useUiStore } from "../../stores/ui-store";
import type { AppSettings } from "../../types/models";

export function SettingsPage() {
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
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

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setCurrentScreen("explorer");
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setCurrentScreen]);

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
    <section className="settings-view" onClick={() => setCurrentScreen("explorer")} role="presentation">
      <div
        aria-labelledby="settings-title"
        aria-modal="true"
        className="settings-panel"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
      >
        <header className="settings-panel-header">
          <div>
            <span className="field-label">Preferences</span>
            <h2 id="settings-title">设置</h2>
          </div>
          <button className="button-secondary settings-close" onClick={() => setCurrentScreen("explorer")} type="button">
            关闭
          </button>
        </header>

        <div className="settings-scroll">
          <section className="settings-section">
            <div className="settings-section-title">
              <h3>项目根目录</h3>
              <code>.skills/</code>
            </div>
            <label className="field-stack">
              <span className="field-label">路径</span>
              <input
                className="detail-save-input"
                onChange={(e) => setProjectRootPath(e.target.value)}
                placeholder="/absolute/path/to/project-root"
                value={projectRootPath}
              />
            </label>
            <div className="settings-action-row">
              <button
                className="button-primary"
                disabled={isSwitching || !projectRootPath.trim()}
                onClick={() => { void switchProjectRoot(projectRootPath); }}
                type="button"
              >
                {isSwitching ? "打开中..." : "打开项目"}
              </button>
            </div>
            {projectRootSuccess ? <div className="inline-banner inline-banner-success">{projectRootSuccess}</div> : null}
            {projectRootError ? <div className="inline-banner inline-banner-error">{projectRootError}</div> : null}
            {settings?.recentProjectRoots?.length ? (
              <div className="settings-recent-list">
                <span className="field-label">最近使用</span>
                {settings.recentProjectRoots.map((item) => (
                  <button
                    className="settings-recent-root"
                    disabled={isSwitching || item.rootPath === settings.currentProjectRoot}
                    key={item.rootPath}
                    onClick={() => { void switchProjectRoot(item.rootPath); }}
                    type="button"
                  >
                    <strong>{item.name}</strong>
                    <span className="mono-text">{item.rootPath}</span>
                  </button>
                ))}
              </div>
            ) : null}

            <div className="settings-create-row">
              <label className="field-stack">
                <span className="field-label">新项目</span>
                <input
                  className="detail-save-input"
                  onChange={(e) => setNewProjectRootName(e.target.value)}
                  placeholder="Research Skills"
                  value={newProjectRootName}
                />
              </label>
              <button
                className="button-secondary"
                disabled={isCreating || !newProjectRootName.trim()}
                onClick={() => { void createAndOpenProjectRoot(); }}
                type="button"
              >
                {isCreating ? "创建中..." : "创建"}
              </button>
            </div>
          </section>

          <section className="settings-section">
            <div className="settings-section-title">
              <h3>本地交接</h3>
              <code>no API key</code>
            </div>
            {settings ? (
              <>
                <div className="agent-config-grid">
                  <label className="field-stack">
                    <span className="field-label">Terminal</span>
                    <input
                      className="detail-save-input"
                      onChange={(e) => setTerminalCommand(e.target.value)}
                      placeholder="open -a Terminal"
                      value={terminalCommand}
                    />
                  </label>
                  <label className="field-stack">
                    <span className="field-label">Editor</span>
                    <input
                      className="detail-save-input"
                      onChange={(e) => setEditorCommand(e.target.value)}
                      placeholder="code"
                      value={editorCommand}
                    />
                  </label>
                  <label className="field-stack">
                    <span className="field-label">Agent</span>
                    <input
                      className="detail-save-input"
                      onChange={(e) => setAgentCommand(e.target.value)}
                      placeholder="codex"
                      value={agentCommand}
                    />
                  </label>
                  <label className="field-stack">
                    <span className="field-label">Global Claude</span>
                    <input
                      className="detail-save-input"
                      onChange={(e) => setGlobalClaudeSkillsDir(e.target.value)}
                      placeholder="~/.claude/skills"
                      value={globalClaudeSkillsDir}
                    />
                  </label>
                  <label className="field-stack agent-config-wide">
                    <span className="field-label">Project Claude</span>
                    <input
                      className="detail-save-input"
                      onChange={(e) => setProjectClaudeSkillsDirName(e.target.value)}
                      placeholder=".claude/skills"
                      value={projectClaudeSkillsDirName}
                    />
                  </label>
                </div>
                <div className="settings-action-row">
                  <button
                    className="button-primary"
                    disabled={isSavingHandoff}
                    onClick={() => { void saveHandoffConfig(); }}
                    type="button"
                  >
                    {isSavingHandoff ? "保存中..." : "保存"}
                  </button>
                </div>
                {handoffSuccess ? <div className="inline-banner inline-banner-success">{handoffSuccess}</div> : null}
                {handoffError ? <div className="inline-banner inline-banner-error">{handoffError}</div> : null}
              </>
            ) : (
              <p className="muted">正在读取...</p>
            )}
          </section>

          <section className="settings-section">
            <div className="settings-section-title">
              <h3>本地状态</h3>
              <code>v{import.meta.env.VITE_APP_VERSION}</code>
            </div>
            {settings ? (
              <dl className="settings-bridge-grid">
                <div>
                  <dt>技能目录</dt>
                  <dd>{settings.skillRootName ?? ".skills"}</dd>
                </div>
                <div>
                  <dt>版本上限</dt>
                  <dd>{settings.formalVersionCap}</dd>
                </div>
                <div>
                  <dt>默认根目录</dt>
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
              <p className="muted">正在读取...</p>
            )}
          </section>
        </div>
      </div>
    </section>
  );
}
