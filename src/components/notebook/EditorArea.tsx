import { useMemo, useState } from "react";
import { useEditorStore } from "../../stores/editor-store";
import { FrontmatterCard } from "./FrontmatterCard";
import { MarkdownPreview } from "./MarkdownPreview";

interface EditorAreaProps {
  packageId: string;
}

function CopyIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <rect x="5" y="2" width="8" height="10" rx="1" />
      <path d="M3 5v8a1 1 0 0 0 1 1h7" />
    </svg>
  );
}

function EditIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M11 2 14 5l-8 8H3v-3Z" />
    </svg>
  );
}

function isMarkdownFile(path: string | null) {
  if (!path) return false;
  const lowerPath = path.toLowerCase();
  return lowerPath.endsWith(".md") || lowerPath.endsWith(".markdown");
}

function countReadableUnits(content: string) {
  const cjkCount = content.match(/[\u3400-\u9fff]/g)?.length ?? 0;
  const latinCount =
    content
      .replace(/[\u3400-\u9fff]/g, " ")
      .match(/[A-Za-z0-9_]+(?:[-'][A-Za-z0-9_]+)*/g)?.length ?? 0;
  return cjkCount + latinCount;
}

export function EditorArea({ packageId }: EditorAreaProps) {
  const [copied, setCopied] = useState(false);
  const currentFilePath = useEditorStore((state) => state.currentFilePath);
  const fileContent = useEditorStore((state) => state.fileContent);
  const mode = useEditorStore((state) => state.mode);
  const isFileLoading = useEditorStore((state) => state.isFileLoading);
  const isSaving = useEditorStore((state) => state.isSaving);
  const isDirty = useEditorStore((state) => state.isDirty);
  const fileError = useEditorStore((state) => state.fileError);
  const saveError = useEditorStore((state) => state.saveError);
  const saveNotice = useEditorStore((state) => state.saveNotice);
  const setMode = useEditorStore((state) => state.setMode);
  const setFileContent = useEditorStore((state) => state.setFileContent);
  const saveFile = useEditorStore((state) => state.saveFile);
  const isMarkdown = isMarkdownFile(currentFilePath);
  const contentCount = useMemo(() => countReadableUnits(fileContent), [fileContent]);
  const fileStateLabel = isSaving ? "保存中" : isDirty ? "未保存" : currentFilePath ? "已同步" : "未打开";
  const fileStateClass = isSaving ? "is-saving" : isDirty ? "is-dirty" : "is-saved";

  async function handleCopy() {
    if (!currentFilePath || !navigator.clipboard) return;
    await navigator.clipboard.writeText(fileContent);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  async function finishEditing() {
    if (isDirty) {
      const saved = await saveFile(packageId);
      if (!saved) return;
    }
    setMode("preview");
  }

  if (!currentFilePath && !fileError) {
    return null;
  }

  return (
    <div className="editor-area">
      <div className="editor-toolbar">
        <div className="editor-toolbar-left">
          <span className="editor-filepath">{currentFilePath ?? "文件未打开"}</span>
          {currentFilePath ? <span className="editor-file-meta">{contentCount} 字词</span> : null}
          <span className={`editor-file-state ${fileStateClass}`}>{fileStateLabel}</span>
        </div>
        <div className="editor-mode-toggle">
          {mode === "edit" ? (
            <button
              className="editor-done-btn"
              disabled={isSaving}
              onClick={() => { void finishEditing(); }}
              type="button"
            >
              {isSaving ? "保存中..." : "完成"}
            </button>
          ) : (
            <button
              className="editor-mode-btn"
              onClick={() => setMode("edit")}
              type="button"
              disabled={!currentFilePath}
            >
              <EditIcon />
              编辑
            </button>
          )}
          <button
            className="editor-copy-btn"
            disabled={!currentFilePath}
            onClick={() => { void handleCopy(); }}
            title="复制当前文件内容"
            type="button"
          >
            {copied ? "✓" : <CopyIcon />}
          </button>
        </div>
      </div>
      {fileError || saveError || saveNotice ? (
        <div className="editor-banner-stack">
          {fileError ? <div className="inline-banner inline-banner-error">{fileError}</div> : null}
          {saveError ? <div className="inline-banner inline-banner-error">{saveError}</div> : null}
          {saveNotice ? <div className="inline-banner inline-banner-success">{saveNotice}</div> : null}
        </div>
      ) : null}
      <div className="editor-content">
        {isFileLoading ? (
          <div className="editor-loading">加载中...</div>
        ) : fileError ? (
          <div className="editor-empty-state">
            <h3>这个文件暂时打不开</h3>
            <p className="muted">可以在左侧重新选择文件，或检查当前项目根目录下的 `.skills/` 是否仍然有效。</p>
          </div>
        ) : !currentFilePath ? (
          <div className="editor-empty-state">
            <h3>还没有打开文件</h3>
            <p className="muted">从左侧文件树选择一个文件，就可以在这里预览或编辑。</p>
          </div>
        ) : mode === "preview" ? (
          <div className={`editor-preview ${isMarkdown ? "is-markdown" : "is-plain"}`}>
            {isMarkdown ? (
              <>
                <FrontmatterCard content={fileContent} filePath={currentFilePath} />
                <MarkdownPreview content={fileContent} />
              </>
            ) : (
              <pre className="editor-preview-text">{fileContent}</pre>
            )}
          </div>
        ) : (
          <textarea
            className="editor-textarea"
            value={fileContent}
            onChange={(e) => setFileContent(e.target.value)}
            spellCheck={false}
          />
        )}
      </div>
    </div>
  );
}
