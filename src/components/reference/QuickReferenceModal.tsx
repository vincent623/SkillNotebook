import { useEffect, useMemo, useState } from "react";
import { exportPackageZip, getPackageReference } from "../../services/tauri-api";
import type {
  PackageExportArtifact,
  PackageReferenceItem,
  PackageReferenceResponse,
  ProjectRoot,
  SkillPackage,
} from "../../types/models";

interface QuickReferenceModalProps {
  onClose: () => void;
  pkg: SkillPackage;
  projectRoot: ProjectRoot;
}

type ReferenceTab = "reference" | "links" | "export";

function itemKindLabel(item: PackageReferenceItem) {
  switch (item.kind) {
    case "command":
      return "命令";
    case "snippet":
      return "引用";
    case "path":
    default:
      return "路径";
  }
}

function itemDisplayLabel(item: PackageReferenceItem) {
  switch (item.id) {
    case "package-path":
      return "包目录";
    case "skill-md-path":
      return "SKILL.md";
    case "markdown-reference":
      return "Markdown 引用";
    case "cli-reference":
      return "CLI 引用";
    case "terminal-command":
      return "终端打开";
    case "global-claude-link":
      return "全局 Claude";
    case "project-claude-link":
      return "项目 Claude";
    default:
      return item.label;
  }
}

export function QuickReferenceModal({ onClose, pkg, projectRoot }: QuickReferenceModalProps) {
  const [tab, setTab] = useState<ReferenceTab>("reference");
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [artifact, setArtifact] = useState<PackageExportArtifact | null>(null);
  const [reference, setReference] = useState<PackageReferenceResponse | null>(null);
  const [referenceError, setReferenceError] = useState<{ packageId: string; message: string } | null>(null);
  const [exportState, setExportState] = useState<"idle" | "exporting" | "error">("idle");
  const [exportError, setExportError] = useState<string | null>(null);
  const activeReference = reference?.packageId === pkg.id ? reference : null;
  const activeReferenceError = referenceError?.packageId === pkg.id ? referenceError.message : null;
  const items = useMemo(() => activeReference?.items ?? [], [activeReference?.items]);
  const visibleItems = useMemo(() => {
    const isClaudeLink = (item: PackageReferenceItem) => item.id.includes("claude-link");
    if (tab === "links") return items.filter(isClaudeLink);
    if (tab === "reference") return items.filter((item) => !isClaudeLink(item));
    return [];
  }, [items, tab]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  useEffect(() => {
    let cancelled = false;
    void getPackageReference(pkg.id)
      .then((next) => {
        if (!cancelled) {
          setReference(next);
          setReferenceError(null);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setReferenceError({
            packageId: pkg.id,
            message: error instanceof Error ? error.message : "加载快速引用失败。",
          });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [pkg.id]);

  async function copyValue(id: string, value: string) {
    setCopiedId(id);
    try {
      await navigator.clipboard?.writeText(value);
    } catch {
      // The desktop runtime and browser preview have different clipboard permission paths.
    }
    window.setTimeout(() => setCopiedId(null), 3000);
  }

  async function createZipExport() {
    setExportState("exporting");
    setExportError(null);
    try {
      const nextArtifact = await exportPackageZip(pkg.id);
      setArtifact(nextArtifact);
      setExportState("idle");
    } catch (error) {
      setExportState("error");
      setExportError(error instanceof Error ? error.message : "导出 zip 失败。");
    }
  }

  return (
    <div className="quick-reference-modal-overlay" onClick={onClose} role="presentation">
      <section
        aria-labelledby="quick-reference-title"
        aria-modal="true"
        className="quick-reference-modal"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
      >
        <header className="quick-reference-modal-header">
          <div>
            <span className="quick-reference-modal-eyebrow">Quick reference</span>
            <h2 id="quick-reference-title">快速引用</h2>
            <p><code>{pkg.slug}</code> · {pkg.name}</p>
          </div>
          <button className="version-modal-close button-secondary" onClick={onClose} type="button">
            关闭
          </button>
        </header>

        <div className="quick-reference-tabs" aria-label="快速引用类型">
          <button className={tab === "reference" ? "is-active" : ""} onClick={() => setTab("reference")} type="button">
            引用
          </button>
          <button className={tab === "links" ? "is-active" : ""} onClick={() => setTab("links")} type="button">
            软链
          </button>
          <button className={tab === "export" ? "is-active" : ""} onClick={() => setTab("export")} type="button">
            ZIP
          </button>
        </div>

        <div className="quick-reference-modal-body">
          <div className="quick-reference-target-summary">
            <span>当前项目</span>
            <strong title={projectRoot.rootPath}>{projectRoot.rootPath}</strong>
          </div>

          {tab === "export" ? (
            <article className="reference-action reference-action-command">
              <div className="reference-action-head">
                <div>
                  <span>导出</span>
                  <h3>Sanitized zip</h3>
                </div>
                <button
                  className="button-secondary reference-copy-btn"
                  disabled={exportState === "exporting"}
                  onClick={() => { void createZipExport(); }}
                  type="button"
                >
                  {exportState === "exporting" ? "导出中..." : "生成"}
                </button>
              </div>
              {artifact ? (
                <pre className="reference-action-value">{artifact.zipPath}</pre>
              ) : (
                <p className="reference-action-note">排除隐藏文件和 notebook.json。</p>
              )}
              {exportError ? <div className="inline-banner inline-banner-error">{exportError}</div> : null}
            </article>
          ) : null}

          {activeReferenceError ? <div className="inline-banner inline-banner-error">{activeReferenceError}</div> : null}
          {!activeReference && !activeReferenceError ? <div className="inline-banner">正在加载快速引用...</div> : null}

          {tab !== "export" ? (
          <div className="reference-action-list">
            {visibleItems.map((item) => (
              <article className={`reference-action reference-action-${item.kind === "command" ? "command" : "path"}`} key={item.id}>
                <div className="reference-action-head">
                  <div>
                    <span>{itemKindLabel(item)}</span>
                    <h3>{itemDisplayLabel(item)}</h3>
                  </div>
                  <button
                    className="button-secondary reference-copy-btn"
                    onClick={() => { void copyValue(item.id, item.value); }}
                    type="button"
                  >
                    {copiedId === item.id ? "已复制" : "复制"}
                  </button>
                </div>
                <pre className="reference-action-value">{item.value}</pre>
              </article>
            ))}
          </div>
          ) : null}
        </div>
      </section>
    </div>
  );
}
