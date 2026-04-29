import { useEffect } from "react";
import type { PackageVersion, PackageVersionDiff, VersionDiffChangeType } from "../../types/models";

interface VersionDiffModalProps {
  open: boolean;
  version: PackageVersion | null;
  diff: PackageVersionDiff | null;
  isLoading: boolean;
  errorMessage: string | null;
  onClose: () => void;
}

function changeLabel(changeType: VersionDiffChangeType): string {
  switch (changeType) {
    case "added":
      return "新增";
    case "removed":
      return "删除";
    case "modified":
      return "修改";
  }
}

export function VersionDiffModal({
  open,
  version,
  diff,
  isLoading,
  errorMessage,
  onClose,
}: VersionDiffModalProps) {
  useEffect(() => {
    if (!open) {
      return undefined;
    }

    const handleKeydown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div className="version-modal-overlay" onClick={onClose} role="presentation">
      <div
        className="version-modal"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="version-diff-title"
      >
        <div className="version-modal-header">
          <div>
            <h3 id="version-diff-title">v{version?.versionNumber ?? "?"} 与当前草稿的差异</h3>
            <p className="muted version-modal-subtitle">
              {version?.snapshotPath ?? "正在加载快照信息..."}
            </p>
          </div>
          <button className="button-secondary version-modal-close" onClick={onClose} type="button">
            关闭
          </button>
        </div>

        <div className="version-modal-body">
          {isLoading ? (
            <div className="version-modal-state muted">正在生成差异视图...</div>
          ) : errorMessage ? (
            <div className="inline-banner inline-banner-error">{errorMessage}</div>
          ) : !diff || diff.entries.length === 0 ? (
            <div className="version-modal-state muted">当前草稿和这个正式版本没有差异。</div>
          ) : (
            <div className="version-diff-list">
              {diff.entries.map((entry) => (
                <section key={entry.path} className="version-diff-entry">
                  <div className="version-diff-entry-header">
                    <span className="version-diff-path">{entry.path}</span>
                    <span className={`version-diff-badge is-${entry.changeType}`}>
                      {changeLabel(entry.changeType)}
                    </span>
                  </div>
                  <pre className="version-diff-pre">{entry.diffText}</pre>
                </section>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
