import { useEffect, useMemo, useState } from "react";
import type { ProjectRoot, SkillPackage } from "../../types/models";

interface ExportUseModalProps {
  onClose: () => void;
  pkg: SkillPackage;
  projectRoot: ProjectRoot;
}

interface ExportAction {
  id: string;
  label: string;
  value: string;
  variant: "path" | "command";
}

function joinPath(...parts: string[]) {
  return parts
    .map((part, index) => {
      if (index === 0) return part.replace(/\/+$/g, "");
      return part.replace(/^\/+|\/+$/g, "");
    })
    .filter(Boolean)
    .join("/");
}

function shellQuote(value: string) {
  return `"${value.replace(/(["\\$`])/g, "\\$1")}"`;
}

function buildExportActions(pkg: SkillPackage, projectRoot: ProjectRoot): ExportAction[] {
  const packagePath = pkg.rootPath;
  const skillMdPath = joinPath(packagePath, "SKILL.md");
  const projectClaudeSkillsDir = joinPath(projectRoot.rootPath, ".claude", "skills");
  const projectClaudeSkillPath = joinPath(projectClaudeSkillsDir, pkg.slug);

  return [
    {
      id: "package-path",
      label: "Package 路径",
      value: packagePath,
      variant: "path",
    },
    {
      id: "skill-md-path",
      label: "SKILL.md 路径",
      value: skillMdPath,
      variant: "path",
    },
    {
      id: "global-claude-link",
      label: "链接到全局 Claude skills",
      value: `mkdir -p ~/.claude/skills && ln -sfn ${shellQuote(packagePath)} ~/.claude/skills/${pkg.slug}`,
      variant: "command",
    },
    {
      id: "project-claude-link",
      label: "链接到当前项目的 Claude skills",
      value: `mkdir -p ${shellQuote(projectClaudeSkillsDir)} && ln -sfn ${shellQuote(packagePath)} ${shellQuote(projectClaudeSkillPath)}`,
      variant: "command",
    },
  ];
}

export function ExportUseModal({ onClose, pkg, projectRoot }: ExportUseModalProps) {
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const actions = useMemo(() => buildExportActions(pkg, projectRoot), [pkg, projectRoot]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  async function copyValue(action: ExportAction) {
    setCopiedId(action.id);
    try {
      await navigator.clipboard?.writeText(action.value);
    } catch {
      // The desktop runtime and browser preview have different clipboard permission paths.
    }
    window.setTimeout(() => setCopiedId(null), 3000);
  }

  return (
    <div className="export-modal-overlay" onClick={onClose} role="presentation">
      <section
        aria-labelledby="export-use-title"
        aria-modal="true"
        className="export-modal"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
      >
        <header className="export-modal-header">
          <div>
            <span className="export-modal-eyebrow">Use locally</span>
            <h2 id="export-use-title">使用 / 导出</h2>
            <p>{pkg.name} · <code>{pkg.slug}</code></p>
          </div>
          <button className="version-modal-close button-secondary" onClick={onClose} type="button">
            关闭
          </button>
        </header>

        <div className="export-modal-body">
          <div className="export-target-summary">
            <span>当前项目</span>
            <strong title={projectRoot.rootPath}>{projectRoot.rootPath}</strong>
          </div>

          <div className="export-action-list">
            {actions.map((action) => (
              <article className={`export-action export-action-${action.variant}`} key={action.id}>
                <div className="export-action-head">
                  <div>
                    <span>{action.variant === "command" ? "命令" : "路径"}</span>
                    <h3>{action.label}</h3>
                  </div>
                  <button
                    className="button-secondary export-copy-btn"
                    onClick={() => { void copyValue(action); }}
                    type="button"
                  >
                    {copiedId === action.id ? "已复制" : "复制"}
                  </button>
                </div>
                <pre className="export-action-value">{action.value}</pre>
              </article>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
