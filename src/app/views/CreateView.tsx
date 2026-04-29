import { useEffect, useMemo, useState } from "react";
import { FileColumnBrowser } from "../../components/browser/FileColumnBrowser";
import { BackButton } from "../../components/common/BackButton";
import { FrontmatterCard } from "../../components/notebook/FrontmatterCard";
import { MarkdownPreview } from "../../components/notebook/MarkdownPreview";
import {
  commitPackagePreview,
  discardPackagePreview,
  generatePackagePreviewFromNl,
  generatePackagePreviewFromSources,
} from "../../services/tauri-api";
import { useProjectStore } from "../../stores/project-store";
import { useUiStore } from "../../stores/ui-store";
import type {
  CreatePackagePreviewResponse,
  PackagePreviewFile,
} from "../../types/models";

type CreatePreviewStatus = "idle" | "generating" | "preview" | "committing" | "error";
type CreateInputMode = "text" | "files";

function normalizePath(path: string) {
  return path.replaceAll("\\", "/");
}

function isMarkdownFile(file: PackagePreviewFile | null) {
  return Boolean(file?.path.toLowerCase().endsWith(".md"));
}

function getDefaultPreviewPath(preview: CreatePackagePreviewResponse) {
  return (
    preview.files.find((file) => file.path.toLowerCase() === "skill.md")?.path ??
    preview.files.find((file) => file.path.toLowerCase().endsWith(".md"))?.path ??
    preview.files[0]?.path ??
    null
  );
}

function discardPreviewQuietly(preview: CreatePackagePreviewResponse) {
  void discardPackagePreview({
    projectRootId: preview.projectRootId,
    previewId: preview.previewId,
  }).catch((error) => {
    console.warn("Failed to discard create preview.", error);
  });
}

export function CreateView() {
  const setCurrentScreen = useUiStore((state) => state.setCurrentScreen);
  const bootstrap = useProjectStore((state) => state.bootstrap);
  const createPrompt = useProjectStore((state) => state.createPrompt);
  const createContext = useProjectStore((state) => state.createContext);
  const setCreatePrompt = useProjectStore((state) => state.setCreatePrompt);
  const setCreateContext = useProjectStore((state) => state.setCreateContext);
  const selectPackage = useProjectStore((state) => state.selectPackage);
  const loadBootstrap = useProjectStore((state) => state.loadBootstrap);
  const [preview, setPreview] = useState<CreatePackagePreviewResponse | null>(null);
  const [selectedPreviewPath, setSelectedPreviewPath] = useState<string | null>(null);
  const [status, setStatus] = useState<CreatePreviewStatus>("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [inputMode, setInputMode] = useState<CreateInputMode>("text");
  const [sourcePaths, setSourcePaths] = useState("");

  const selectedPreviewFile = useMemo(() => {
    if (!preview || !selectedPreviewPath) return null;
    const selectedPath = normalizePath(selectedPreviewPath);
    return preview.files.find((file) => normalizePath(file.path) === selectedPath) ?? null;
  }, [preview, selectedPreviewPath]);

  const isGenerating = status === "generating";
  const isCommitting = status === "committing";
  const sourcePathList = useMemo(
    () => sourcePaths.split(/\r?\n/).map((path) => path.trim()).filter(Boolean),
    [sourcePaths],
  );
  const canGenerate = Boolean(
    bootstrap &&
      !isGenerating &&
      !isCommitting &&
      (inputMode === "text" ? createPrompt.trim() : sourcePathList.length > 0),
  );

  useEffect(() => {
    if (!preview) {
      return undefined;
    }

    return () => discardPreviewQuietly(preview);
  }, [preview]);

  const clearPreview = () => {
    setPreview(null);
    setSelectedPreviewPath(null);
    setStatus("idle");
    setErrorMessage(null);
  };

  const handleGeneratePreview = async () => {
    if (!bootstrap) {
      setStatus("error");
      setErrorMessage("项目根目录还在加载。");
      return;
    }

    const prompt = createPrompt.trim();
    const context = createContext.trim();
    if (inputMode === "text" && !prompt) {
      setStatus("error");
      setErrorMessage("先写下这个技能要做什么。");
      return;
    }
    if (inputMode === "files" && sourcePathList.length === 0) {
      setStatus("error");
      setErrorMessage("至少输入一个本地文件或目录路径。");
      return;
    }

    setStatus("generating");
    setErrorMessage(null);

    try {
      const nextPreview = inputMode === "files"
        ? await generatePackagePreviewFromSources({
            projectRootId: bootstrap.projectRoot.id,
            sourcePaths: sourcePathList,
            prompt: prompt || null,
            context: context || null,
          })
        : await generatePackagePreviewFromNl({
            projectRootId: bootstrap.projectRoot.id,
            prompt,
            context: context || null,
          });
      setPreview(nextPreview);
      setSelectedPreviewPath(getDefaultPreviewPath(nextPreview));
      setStatus("preview");
    } catch (error) {
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : "生成预览失败。");
    }
  };

  const handleCommitPreview = async () => {
    if (!preview) return;

    setStatus("committing");
    setErrorMessage(null);

    try {
      const result = await commitPackagePreview({
        projectRootId: preview.projectRootId,
        previewId: preview.previewId,
      });
      selectPackage(result.packageId);
      await loadBootstrap();
      setCreatePrompt("");
      setCreateContext("");
      setSourcePaths("");
      setPreview(null);
      setSelectedPreviewPath(null);
      setStatus("idle");
      setCurrentScreen("notebook");
    } catch (error) {
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : "保存预览失败。");
    }
  };

  return (
    <section className="create-view">
      <BackButton />
      <div className="create-flow">
        <aside className="content-card create-form-panel">
          <div className="create-heading">
            <span className="field-label">Create</span>
            <h2 className="create-title">新建技能包</h2>
          </div>
          <div className="create-mode-tabs" aria-label="创建来源">
            <button
              className={inputMode === "text" ? "is-active" : ""}
              disabled={isGenerating || isCommitting}
              onClick={() => {
                setInputMode("text");
                if (preview) clearPreview();
              }}
              type="button"
            >
              文本
            </button>
            <button
              className={inputMode === "files" ? "is-active" : ""}
              disabled={isGenerating || isCommitting}
              onClick={() => {
                setInputMode("files");
                if (preview) clearPreview();
              }}
              type="button"
            >
              文件/目录
            </button>
            <button disabled type="button">
              URL
            </button>
          </div>
          {inputMode === "files" ? (
            <label className="field-stack">
              <span className="field-label">本地路径（每行一个）</span>
              <textarea
                className="form-textarea form-textarea-sm create-source-paths"
                onChange={(event) => {
                  setSourcePaths(event.target.value);
                  if (preview) clearPreview();
                }}
                placeholder={`${bootstrap?.projectRoot.rootPath ?? "/absolute/path"}/notes\n./relative/path/from/project-root`}
                rows={5}
                value={sourcePaths}
              />
              <span className="create-field-hint">
                支持文件或目录。相对路径按当前项目根目录解析；绝对路径会在本机读取。PDF、docx、图片等二进制只记录文件名和大小，UTF-8 文本会摘录进 source-inventory。
              </span>
            </label>
          ) : null}
          <label className="field-stack">
            <span className="field-label">
              {inputMode === "files" ? "生成目标（可选）" : "这个技能要做什么？"}
            </span>
            <textarea
              className="form-textarea"
              onChange={(event) => {
                setCreatePrompt(event.target.value);
                if (preview) clearPreview();
              }}
              placeholder={inputMode === "files"
                ? "例：基于这些访谈记录，生成一个洞察提炼 Skill。"
                : "例：将客户访谈笔记提炼为结构化洞察和行动建议。"}
              rows={inputMode === "files" ? 4 : 6}
              value={createPrompt}
            />
          </label>
          <label className="field-stack">
            <span className="field-label">补充说明（可选）</span>
            <textarea
              className="form-textarea form-textarea-sm"
              onChange={(event) => {
                setCreateContext(event.target.value);
                if (preview) clearPreview();
              }}
              placeholder="约束条件、输出格式、来源格式..."
              rows={4}
              value={createContext}
            />
          </label>
          {errorMessage ? (
            <div className="inline-banner inline-banner-error">{errorMessage}</div>
          ) : null}
          <div className="create-form-actions">
            {preview ? (
              <button
                className="button-secondary"
                disabled={isGenerating || isCommitting}
                onClick={clearPreview}
                type="button"
              >
                返回修改
              </button>
            ) : null}
            <button
              className="button-secondary"
              disabled={!canGenerate}
              onClick={handleGeneratePreview}
              type="button"
            >
              {isGenerating ? "生成中..." : preview ? "重新生成" : "生成预览"}
            </button>
            <button
              className="button-primary"
              disabled={!preview || isCommitting || isGenerating}
              onClick={handleCommitPreview}
              type="button"
            >
              {isCommitting ? "保存中..." : "确认保存"}
            </button>
          </div>
        </aside>

        <section className={`create-preview-panel ${preview ? "has-preview" : ""}`}>
          {preview ? (
            <>
              <header className="create-preview-toolbar">
                <div className="create-preview-summary">
                  <span className="field-label">Preview</span>
                  <h3>{preview.name}</h3>
                  <p>{preview.description}</p>
                  <div className="create-preview-tags">
                    {preview.tags.map((tag) => (
                      <span key={tag}>{tag}</span>
                    ))}
                  </div>
                </div>
                <dl className="create-preview-meta">
                  <div>
                    <dt>slug</dt>
                    <dd>{preview.slug}</dd>
                  </div>
                  <div>
                    <dt>generator</dt>
                    <dd>{preview.generatorUsed}</dd>
                  </div>
                </dl>
              </header>
              <div className="create-preview-layout">
                <div className="create-preview-browser">
                  <FileColumnBrowser
                    currentFilePath={selectedPreviewPath}
                    entries={preview.fileTree}
                    errorMessage={null}
                    isLoading={false}
                    onSelectFile={setSelectedPreviewPath}
                    packageSlug={preview.slug}
                  />
                </div>
                <div className="create-preview-content">
                  {selectedPreviewFile ? (
                    <>
                      <div className="create-preview-filebar">
                        <span>{selectedPreviewFile.path}</span>
                        <strong>{selectedPreviewFile.encoding}</strong>
                      </div>
                      {isMarkdownFile(selectedPreviewFile) ? (
                        <div className="editor-preview is-markdown create-preview-markdown">
                          <FrontmatterCard
                            content={selectedPreviewFile.content}
                            filePath={selectedPreviewFile.path}
                          />
                          <MarkdownPreview content={selectedPreviewFile.content} />
                        </div>
                      ) : (
                        <pre className="create-preview-pre">
                          <code>{selectedPreviewFile.content}</code>
                        </pre>
                      )}
                    </>
                  ) : (
                    <div className="create-preview-empty">暂无选中文件。</div>
                  )}
                </div>
              </div>
            </>
          ) : (
            <div className="create-preview-empty is-large">
              <strong>等待预览</strong>
              <span>尚无预览文件。</span>
            </div>
          )}
        </section>
      </div>
    </section>
  );
}
