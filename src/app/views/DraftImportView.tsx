import { useEffect, useMemo, useState } from "react";
import { BackButton } from "../../components/common/BackButton";
import {
  discardDraft,
  importDraft,
  importPackage,
  listDrafts,
  startDraft,
} from "../../services/tauri-api";
import { useProjectStore } from "../../stores/project-store";
import { useUiStore } from "../../stores/ui-store";
import type { DraftWorkspace } from "../../types/models";

type DraftImportMode = "import" | "draft";
type SubmitState = "idle" | "submitting" | "success" | "error";

function normalizePathInput(path: string) {
  let normalized = path.trim();
  while (normalized.length >= 2) {
    const first = normalized[0];
    const last = normalized[normalized.length - 1];
    if ((first === "\"" && last === "\"") || (first === "'" && last === "'")) {
      normalized = normalized.slice(1, -1).trim();
      continue;
    }
    break;
  }
  if (normalized.startsWith("file://")) {
    try {
      normalized = decodeURI(normalized.replace(/^file:\/\//, ""));
    } catch {
      normalized = normalized.replace(/^file:\/\//, "");
    }
  }
  return normalized.replace(/\\([\\ "'():])/g, "$1");
}

function formatDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function DraftImportView() {
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const selectPackage = useProjectStore((state) => state.selectPackage);
  const loadBootstrap = useProjectStore((state) => state.loadBootstrap);
  const [mode, setMode] = useState<DraftImportMode>("import");
  const [status, setStatus] = useState<SubmitState>("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [sourcePath, setSourcePath] = useState("");
  const [importSlug, setImportSlug] = useState("");
  const [runEvalOnImport, setRunEvalOnImport] = useState(true);
  const [draftPrompt, setDraftPrompt] = useState("");
  const [draftSourcePaths, setDraftSourcePaths] = useState("");
  const [draftSourceUrl, setDraftSourceUrl] = useState("");
  const [agentCommand, setAgentCommand] = useState("codex");
  const [drafts, setDrafts] = useState<DraftWorkspace[]>([]);
  const [lastDraft, setLastDraft] = useState<DraftWorkspace | null>(null);

  const sourcePathList = useMemo(
    () => draftSourcePaths.split(/\r?\n/).map(normalizePathInput).filter(Boolean),
    [draftSourcePaths],
  );
  const canImport = Boolean(bootstrap && sourcePath.trim() && status !== "submitting");
  const canStartDraft = Boolean(
    bootstrap &&
      status !== "submitting" &&
      (draftPrompt.trim() || sourcePathList.length > 0 || draftSourceUrl.trim()),
  );

  useEffect(() => {
    let cancelled = false;
    void listDrafts()
      .then((items) => {
        if (!cancelled) setDrafts(items);
      })
      .catch(() => {
        if (!cancelled) setDrafts([]);
      });
    return () => {
      cancelled = true;
    };
  }, [lastDraft, status]);

  async function refreshDrafts() {
    try {
      setDrafts(await listDrafts());
    } catch {
      setDrafts([]);
    }
  }

  async function handleImportPackage() {
    if (!bootstrap) return;
    setStatus("submitting");
    setErrorMessage(null);
    setSuccessMessage(null);
    try {
      const result = await importPackage({
        projectRootId: bootstrap.projectRoot.id,
        sourcePath: normalizePathInput(sourcePath),
        slug: importSlug.trim() || null,
        runEval: runEvalOnImport,
      });
      selectPackage(result.packageId);
      await loadBootstrap();
      setSuccessMessage(`已导入 ${result.slug}。`);
      setCurrentScreen("notebook");
    } catch (error) {
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : "导入失败。");
      return;
    }
    setStatus("success");
  }

  async function handleStartDraft() {
    if (!bootstrap) return;
    setStatus("submitting");
    setErrorMessage(null);
    setSuccessMessage(null);
    try {
      const draft = await startDraft({
        projectRootId: bootstrap.projectRoot.id,
        prompt: draftPrompt.trim() || null,
        sourcePaths: sourcePathList.length > 0 ? sourcePathList : null,
        sourceUrl: draftSourceUrl.trim() || null,
        preferredAgentCommand: agentCommand.trim() || "codex",
      });
      setLastDraft(draft);
      setSuccessMessage(`已创建草稿 ${draft.draftId}。`);
      await refreshDrafts();
      setStatus("success");
    } catch (error) {
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : "创建草稿失败。");
    }
  }

  async function handleImportDraft(draft: DraftWorkspace) {
    if (!bootstrap) return;
    setStatus("submitting");
    setErrorMessage(null);
    setSuccessMessage(null);
    try {
      const result = await importDraft({
        projectRootId: bootstrap.projectRoot.id,
        draftId: draft.draftId,
        runEval: runEvalOnImport,
      });
      selectPackage(result.packageId);
      await loadBootstrap();
      setSuccessMessage(`已导入草稿 ${result.slug}。`);
      setCurrentScreen("notebook");
    } catch (error) {
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : "导入草稿失败。");
      return;
    }
    setStatus("success");
  }

  async function handleDiscardDraft(draft: DraftWorkspace) {
    if (!bootstrap) return;
    setStatus("submitting");
    setErrorMessage(null);
    try {
      await discardDraft({
        projectRootId: bootstrap.projectRoot.id,
        draftId: draft.draftId,
      });
      if (lastDraft?.draftId === draft.draftId) setLastDraft(null);
      await refreshDrafts();
      setStatus("idle");
    } catch (error) {
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : "丢弃草稿失败。");
    }
  }

  async function copyText(value: string) {
    try {
      await navigator.clipboard?.writeText(value);
      setSuccessMessage("已复制。");
    } catch {
      setSuccessMessage("已准备好复制内容。");
    }
  }

  return (
    <section className="draft-import-view">
      <BackButton />
      <div className="draft-import-flow">
        <aside className="content-card draft-import-form-panel">
          <div className="draft-import-heading">
            <span className="field-label">Draft / Import</span>
            <h2 className="draft-import-title">导入或新建草稿</h2>
          </div>
          <div className="draft-import-mode-tabs" aria-label="草稿和导入">
            <button className={mode === "import" ? "is-active" : ""} onClick={() => setMode("import")} type="button">
              导入 Skill
            </button>
            <button className={mode === "draft" ? "is-active" : ""} onClick={() => setMode("draft")} type="button">
              新建草稿
            </button>
          </div>

          {mode === "import" ? (
            <>
              <label className="field-stack">
                <span className="field-label">候选 skill 目录</span>
                <input
                  className="detail-save-input"
                  onChange={(event) => setSourcePath(event.target.value)}
                  placeholder={`${bootstrap?.projectRoot.rootPath ?? "/absolute/path"}/some-skill`}
                  value={sourcePath}
                />
                <span className="draft-import-field-hint">目录必须包含 SKILL.md。导入后会复制到当前项目的 .skills/。</span>
              </label>
              <label className="field-stack">
                <span className="field-label">slug（可选）</span>
                <input
                  className="detail-save-input"
                  onChange={(event) => setImportSlug(event.target.value)}
                  placeholder="留空则使用目录名"
                  value={importSlug}
                />
              </label>
              <label className="settings-checkbox-row">
                <input
                  checked={runEvalOnImport}
                  onChange={(event) => setRunEvalOnImport(event.target.checked)}
                  type="checkbox"
                />
                导入后立即运行 eval
              </label>
              <div className="draft-import-form-actions">
                <button
                  className="button-primary"
                  disabled={!canImport}
                  onClick={() => { void handleImportPackage(); }}
                  type="button"
                >
                  {status === "submitting" ? "导入中..." : "导入到 .skills/"}
                </button>
              </div>
            </>
          ) : (
            <>
              <label className="field-stack">
                <span className="field-label">草稿目标</span>
                <textarea
                  className="form-textarea"
                  onChange={(event) => setDraftPrompt(event.target.value)}
                  placeholder="例：把会议纪要整理为负责人、截止日期、风险和行动项。"
                  rows={4}
                  value={draftPrompt}
                />
              </label>
              <label className="field-stack">
                <span className="field-label">本地来源路径（可选，每行一个）</span>
                <textarea
                  className="form-textarea form-textarea-sm"
                  onChange={(event) => setDraftSourcePaths(event.target.value)}
                  placeholder="/absolute/path/to/source"
                  rows={4}
                  value={draftSourcePaths}
                />
              </label>
              <label className="field-stack">
                <span className="field-label">URL 来源（可选）</span>
                <input
                  className="detail-save-input"
                  onChange={(event) => setDraftSourceUrl(event.target.value)}
                  placeholder="https://example.com/source"
                  value={draftSourceUrl}
                />
              </label>
              <label className="field-stack">
                <span className="field-label">外部 Agent 命令</span>
                <input
                  className="detail-save-input"
                  onChange={(event) => setAgentCommand(event.target.value)}
                  placeholder="codex"
                  value={agentCommand}
                />
              </label>
              <div className="draft-import-form-actions">
                <button
                  className="button-primary"
                  disabled={!canStartDraft}
                  onClick={() => { void handleStartDraft(); }}
                  type="button"
                >
                  {status === "submitting" ? "创建中..." : "创建草稿工作区"}
                </button>
              </div>
            </>
          )}

          {successMessage ? <div className="inline-banner inline-banner-success">{successMessage}</div> : null}
          {errorMessage ? <div className="inline-banner inline-banner-error">{errorMessage}</div> : null}
        </aside>

        <section className="draft-import-preview-panel has-preview">
          {lastDraft ? (
            <article className="content-card">
              <span className="field-label">Last draft</span>
              <h3>{lastDraft.intendedSlug}</h3>
              <p className="muted">{lastDraft.sourceSummary}</p>
              <div className="reference-action-list" style={{ marginTop: 12 }}>
                <article className="reference-action reference-action-path">
                  <div className="reference-action-head">
                    <div>
                      <span>路径</span>
                      <h3>Draft workspace</h3>
                    </div>
                    <button className="button-secondary reference-copy-btn" onClick={() => { void copyText(lastDraft.draftPath); }} type="button">
                      复制
                    </button>
                  </div>
                  <pre className="reference-action-value">{lastDraft.draftPath}</pre>
                </article>
                <article className="reference-action reference-action-command">
                  <div className="reference-action-head">
                    <div>
                      <span>命令</span>
                      <h3>交给外部 Agent</h3>
                    </div>
                    <button className="button-secondary reference-copy-btn" onClick={() => { void copyText(lastDraft.suggestedCommand); }} type="button">
                      复制
                    </button>
                  </div>
                  <pre className="reference-action-value">{lastDraft.suggestedCommand}</pre>
                </article>
              </div>
            </article>
          ) : (
            <div className="workbench-empty-pane is-large">
              <strong>Skill Notebook 不在这里生成 skill</strong>
              <span>这里负责导入候选包，或创建临时草稿目录并交给 Claude/Codex/OpenClaw。</span>
            </div>
          )}

          <article className="content-card" style={{ marginTop: 16 }}>
            <span className="field-label">Active drafts</span>
            <h3>草稿工作区</h3>
            {drafts.length === 0 ? (
              <p className="muted" style={{ marginTop: 8 }}>暂无草稿。</p>
            ) : (
              <div className="reference-action-list" style={{ marginTop: 12 }}>
                {drafts.map((draft) => (
                  <article className="reference-action reference-action-command" key={draft.draftId}>
                    <div className="reference-action-head">
                      <div>
                        <span>{formatDate(draft.createdAt)}</span>
                        <h3>{draft.intendedSlug}</h3>
                      </div>
                      <div className="settings-action-row">
                        <button className="button-secondary reference-copy-btn" onClick={() => { void copyText(draft.suggestedCommand); }} type="button">
                          复制命令
                        </button>
                        <button className="button-secondary reference-copy-btn" onClick={() => { void handleImportDraft(draft); }} type="button">
                          导入
                        </button>
                        <button className="button-secondary reference-copy-btn" onClick={() => { void handleDiscardDraft(draft); }} type="button">
                          丢弃
                        </button>
                      </div>
                    </div>
                    <pre className="reference-action-value">{draft.draftPath}</pre>
                  </article>
                ))}
              </div>
            )}
          </article>
        </section>
      </div>
    </section>
  );
}
