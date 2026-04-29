import { useState } from "react";
import type { FileEntry } from "../../types/models";

interface FileTreeProps {
  entries: FileEntry[];
  currentFilePath: string | null;
  onSelectFile: (path: string) => void;
  isLoading: boolean;
  errorMessage: string | null;
}

function TreeNode({
  entry,
  currentFilePath,
  onSelectFile,
  depth,
}: {
  entry: FileEntry;
  currentFilePath: string | null;
  onSelectFile: (path: string) => void;
  depth: number;
}) {
  const [expanded, setExpanded] = useState(true);
  const isActive = currentFilePath === entry.path;

  if (entry.isDirectory) {
    const hasChildren = entry.children && entry.children.length > 0;
    return (
      <div className="tree-dir">
        <button
          className={`tree-item tree-dir-toggle ${!hasChildren ? "tree-empty-dir" : ""}`}
          onClick={() => hasChildren && setExpanded(!expanded)}
          style={{ paddingLeft: `${12 + depth * 14}px` }}
          type="button"
          disabled={!hasChildren}
        >
          <span className="tree-arrow">{hasChildren ? (expanded ? "▾" : "▸") : "▸"}</span>
          <span className="tree-name">{entry.name}/</span>
        </button>
        {expanded && hasChildren ? (
          <div className="tree-children">
            {entry.children!.map((child) => (
              <TreeNode
                key={child.path}
                entry={child}
                currentFilePath={currentFilePath}
                onSelectFile={onSelectFile}
                depth={depth + 1}
              />
            ))}
          </div>
        ) : null}
      </div>
    );
  }

  return (
    <button
      className={`tree-item tree-file ${isActive ? "is-active" : ""}`}
      onClick={() => onSelectFile(entry.path)}
      style={{ paddingLeft: `${12 + depth * 14}px` }}
      type="button"
    >
      <span className="tree-name">{entry.name}</span>
    </button>
  );
}

export function FileTree({
  entries,
  currentFilePath,
  onSelectFile,
  isLoading,
  errorMessage,
}: FileTreeProps) {
  if (isLoading) {
    return <div className="file-tree-state muted">正在读取文件结构...</div>;
  }

  if (errorMessage) {
    return <div className="file-tree-state file-tree-state-error">{errorMessage}</div>;
  }

  if (entries.length === 0) {
    return <div className="file-tree-state muted">这个 package 里还没有可编辑文件。</div>;
  }

  return (
    <div className="file-tree">
      {entries.map((entry) => (
        <TreeNode
          key={entry.path}
          entry={entry}
          currentFilePath={currentFilePath}
          onSelectFile={onSelectFile}
          depth={0}
        />
      ))}
    </div>
  );
}
