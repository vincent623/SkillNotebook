import { useEffect, useState } from "react";
import { createProjectRoot, getSettings, openProjectRoot, updateSettings } from "../../services/tauri-api";
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
  const [runtimeMode, setRuntimeMode] = useState("auto");
  const [agentProvider, setAgentProvider] = useState("openai-compatible");
  const [agentBaseUrl, setAgentBaseUrl] = useState("");
  const [agentApiKey, setAgentApiKey] = useState("");
  const [agentModel, setAgentModel] = useState("");
  const [agentNodeBinary, setAgentNodeBinary] = useState("node");
  const [agentSidecarScript, setAgentSidecarScript] = useState("");
  const [agentTimeoutSecs, setAgentTimeoutSecs] = useState("300");
  const [agentRetryAttempts, setAgentRetryAttempts] = useState("3");
  const [agentConfigError, setAgentConfigError] = useState<string | null>(null);
  const [agentConfigSuccess, setAgentConfigSuccess] = useState<string | null>(null);
  const [isSavingAgentConfig, setIsSavingAgentConfig] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const next = await getSettings();
        if (cancelled) return;
        setSettings(next);
        populateAgentForm(next);
        setWorkspacePath(next.currentProjectRoot || (bootstrap?.projectRoot.rootPath ?? ""));
      } catch (error) {
        if (!cancelled) setWorkspaceError(error instanceof Error ? error.message : "加载设置失败");
      }
    }
    void load();
    return () => { cancelled = true; };
  }, [bootstrap?.projectRoot.rootPath]);

  function populateAgentForm(next: AppSettings) {
    const bridge = next.creationBridge;
    setRuntimeMode(bridge.mode || "auto");
    setAgentProvider(bridge.agentProvider || "openai-compatible");
    setAgentBaseUrl(bridge.agentBaseUrl ?? "");
    setAgentApiKey("");
    setAgentModel(bridge.agentModel ?? "");
    setAgentNodeBinary(bridge.piNodeBinary || "node");
    setAgentSidecarScript(bridge.piSidecarScript ?? "");
    setAgentTimeoutSecs(String(bridge.agentTimeoutSecs || 300));
    setAgentRetryAttempts(String(bridge.agentRetryAttempts || 3));
  }

  async function switchProjectRoot(path: string) {
    setIsSwitching(true);
    setWorkspaceError(null);
    setWorkspaceSuccess(null);
    try {
      const projectRoot = await openProjectRoot(path.trim());
      await loadBootstrap();
      const next = await getSettings();
      setSettings(next);
      populateAgentForm(next);
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
      populateAgentForm(next);
      setWorkspacePath(projectRoot.rootPath);
      setNewProjectRootName("");
      setWorkspaceSuccess(`已创建并打开 ${projectRoot.rootPath}`);
    } catch (error) {
      setWorkspaceError(error instanceof Error ? error.message : "创建失败");
    } finally {
      setIsCreating(false);
    }
  }

  async function saveAgentRuntimeConfig(clearApiKey = false) {
    setIsSavingAgentConfig(true);
    setAgentConfigError(null);
    setAgentConfigSuccess(null);
    try {
      const timeoutSecs = Number.parseInt(agentTimeoutSecs, 10);
      const retryAttempts = Number.parseInt(agentRetryAttempts, 10);
      if (!Number.isFinite(timeoutSecs) || timeoutSecs <= 0) {
        throw new Error("Agent 超时必须是大于 0 的秒数。");
      }
      if (!Number.isFinite(retryAttempts) || retryAttempts <= 0) {
        throw new Error("Agent 重试次数必须大于 0。");
      }

      const apiKey = agentApiKey.trim();
      const payload = {
        agentRuntime: {
          mode: runtimeMode,
          provider: agentProvider,
          baseUrl: agentBaseUrl,
          model: agentModel,
          nodeBinary: agentNodeBinary,
          sidecarScript: agentSidecarScript,
          timeoutSecs,
          retryAttempts,
          clearApiKey,
          ...(apiKey ? { apiKey } : {}),
        },
      };
      const next = await updateSettings(payload);
      setSettings(next);
      populateAgentForm(next);
      setAgentConfigSuccess(clearApiKey ? "Agent API key 已清除。" : "Agent runtime 配置已保存并刷新。");
    } catch (error) {
      setAgentConfigError(error instanceof Error ? error.message : "保存 Agent runtime 配置失败");
    } finally {
      setIsSavingAgentConfig(false);
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
          <h3>Agent Runtime 配置</h3>
          {settings ? (
            <>
              <div className="agent-config-grid" style={{ marginTop: 12 }}>
                <label className="field-stack">
                  <span className="field-label">运行时</span>
                  <select
                    className="detail-save-input"
                    onChange={(e) => setRuntimeMode(e.target.value)}
                    value={runtimeMode}
                  >
                    <option value="auto">auto</option>
                    <option value="pi_sidecar">pi_sidecar</option>
                    <option value="skill_create">skill_create</option>
                    <option value="claude_cli">claude_cli</option>
                    <option value="template">template</option>
                  </select>
                </label>
                <label className="field-stack">
                  <span className="field-label">Provider</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setAgentProvider(e.target.value)}
                    placeholder="openai-compatible"
                    value={agentProvider}
                  />
                </label>
                <label className="field-stack agent-config-wide">
                  <span className="field-label">Base URL</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setAgentBaseUrl(e.target.value)}
                    placeholder="https://api.example.com/v1"
                    value={agentBaseUrl}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">Model</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setAgentModel(e.target.value)}
                    placeholder="model-id"
                    value={agentModel}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">API Key</span>
                  <input
                    autoComplete="off"
                    className="detail-save-input"
                    onChange={(e) => setAgentApiKey(e.target.value)}
                    placeholder={settings.creationBridge.agentApiKeyConfigured ? "已保存；留空则保留" : "粘贴 API key"}
                    type="password"
                    value={agentApiKey}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">Node binary</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setAgentNodeBinary(e.target.value)}
                    placeholder="node"
                    value={agentNodeBinary}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">Sidecar script</span>
                  <input
                    className="detail-save-input"
                    onChange={(e) => setAgentSidecarScript(e.target.value)}
                    placeholder="留空使用内置 sidecar"
                    value={agentSidecarScript}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">Timeout seconds</span>
                  <input
                    className="detail-save-input"
                    inputMode="numeric"
                    onChange={(e) => setAgentTimeoutSecs(e.target.value)}
                    value={agentTimeoutSecs}
                  />
                </label>
                <label className="field-stack">
                  <span className="field-label">Retry attempts</span>
                  <input
                    className="detail-save-input"
                    inputMode="numeric"
                    onChange={(e) => setAgentRetryAttempts(e.target.value)}
                    value={agentRetryAttempts}
                  />
                </label>
              </div>
              <p className="muted" style={{ marginTop: 8 }}>
                保存到 <span className="mono-text">{settings.settingsPath ?? "本机设置文件"}</span>。环境变量仍会覆盖这里的配置。
              </p>
              <div className="settings-action-row">
                <button
                  className="button-primary"
                  disabled={isSavingAgentConfig}
                  onClick={() => { void saveAgentRuntimeConfig(false); }}
                  type="button"
                >
                  {isSavingAgentConfig ? "保存中..." : "保存并检查"}
                </button>
                <button
                  className="button-secondary"
                  disabled={isSavingAgentConfig || !settings.creationBridge.agentApiKeyConfigured}
                  onClick={() => { void saveAgentRuntimeConfig(true); }}
                  type="button"
                >
                  清除 API key
                </button>
              </div>
              {agentConfigSuccess ? <div className="inline-banner inline-banner-success">{agentConfigSuccess}</div> : null}
              {agentConfigError ? <div className="inline-banner inline-banner-error">{agentConfigError}</div> : null}
            </>
          ) : (
            <p className="muted" style={{ marginTop: 8 }}>正在读取 Agent runtime 配置...</p>
          )}
        </div>

        <div className="content-card">
          <h3>创建桥接状态</h3>
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
                <dt>Pi runtime</dt>
                <dd>{settings.creationBridge.piSidecarAvailable ? "可用" : settings.creationBridge.piSidecarConfigured ? "配置未就绪" : "未配置"}</dd>
              </div>
              <div>
                <dt>Agent provider</dt>
                <dd>{settings.creationBridge.agentProvider}</dd>
              </div>
              <div>
                <dt>Agent model</dt>
                <dd>{settings.creationBridge.agentModel ?? "未配置"}</dd>
              </div>
              <div>
                <dt>Agent API</dt>
                <dd>
                  Base URL {settings.creationBridge.agentBaseUrlConfigured ? "已配置" : "未配置"} · API key {settings.creationBridge.agentApiKeyConfigured ? "已配置" : "未配置"}
                </dd>
              </div>
              <div>
                <dt>Pi Node</dt>
                <dd>
                  {settings.creationBridge.piNodeResolvedPath ? (
                    <code className="settings-bridge-path">{settings.creationBridge.piNodeResolvedPath}</code>
                  ) : (
                    `${settings.creationBridge.piNodeBinary} 未解析`
                  )}
                </dd>
              </div>
              <div>
                <dt>Pi sidecar</dt>
                <dd>
                  {settings.creationBridge.piSidecarScriptPath ? (
                    <code className="settings-bridge-path">{settings.creationBridge.piSidecarScriptPath}</code>
                  ) : (
                    "未找到脚本"
                  )}
                </dd>
              </div>
              <div>
                <dt>Agent 超时</dt>
                <dd>{settings.creationBridge.agentTimeoutSecs}s · 重试 {settings.creationBridge.agentRetryAttempts} 次</dd>
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
                <dt>Claude 重试</dt>
                <dd>
                  {settings.creationBridge.claudeRetryAttempts} 次 · 间隔 {settings.creationBridge.claudeRetryBackoffSecs}s
                </dd>
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
            Skill Notebook v{import.meta.env.VITE_APP_VERSION} — macOS 本地优先的 skill 仓库与版本管理工具。
          </p>
          <p className="muted" style={{ marginTop: 4 }}>
            Tauri 2 + Rust + React + TypeScript · Apple Silicon 适配
          </p>
        </div>
      </div>
    </section>
  );
}
