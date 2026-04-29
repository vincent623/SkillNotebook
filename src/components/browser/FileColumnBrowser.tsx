import { useMemo, useState } from "react";
import type { FileEntry } from "../../types/models";

interface FileColumnBrowserProps {
  entries: FileEntry[];
  currentFilePath: string | null;
  errorMessage: string | null;
  isLoading: boolean;
  packageSlug: string;
  onSelectFile: (path: string) => void;
}

interface FileColumn {
  label: string;
  path: string | null;
  entries: FileEntry[];
}

function isHiddenEntry(entry: FileEntry) {
  return entry.name.startsWith(".") || entry.name === "notebook.json";
}

function isSkillFile(entry: FileEntry) {
  return !entry.isDirectory && entry.name.toLowerCase() === "skill.md";
}

function normalizePath(path: string) {
  return path.replaceAll("\\", "/");
}

function visibleSortedEntries(entries: FileEntry[]): FileEntry[] {
  return entries
    .filter((entry) => !isHiddenEntry(entry))
    .map((entry) => ({
      ...entry,
      path: normalizePath(entry.path),
      children: entry.children ? visibleSortedEntries(entry.children) : entry.children,
    }))
    .sort((left, right) => {
      if (left.isDirectory !== right.isDirectory) {
        return left.isDirectory ? -1 : 1;
      }
      if (isSkillFile(left) !== isSkillFile(right)) {
        return isSkillFile(left) ? -1 : 1;
      }
      return left.name.localeCompare(right.name, "zh-CN", { sensitivity: "base" });
    });
}

function findEntry(entries: FileEntry[], path: string | null): FileEntry | null {
  if (!path) return null;
  const target = normalizePath(path);

  for (const entry of entries) {
    if (entry.path === target) return entry;
    if (entry.children) {
      const found = findEntry(entry.children, target);
      if (found) return found;
    }
  }

  return null;
}

function getParentPaths(path: string | null) {
  if (!path) return [];
  const parts = normalizePath(path).split("/").filter(Boolean);
  if (parts.length <= 1) return [];

  const parents: string[] = [];
  for (let index = 1; index < parts.length; index += 1) {
    parents.push(parts.slice(0, index).join("/"));
  }
  return parents;
}

function getDirectoryChain(entries: FileEntry[], activePath: string | null) {
  const activeEntry = findEntry(entries, activePath);
  const parentPaths = getParentPaths(activePath).filter((path) => findEntry(entries, path)?.isDirectory);

  if (activeEntry?.isDirectory) {
    parentPaths.push(activeEntry.path);
  }

  return parentPaths;
}

function getColumnLabel(path: string | null, packageSlug: string) {
  if (!path) return packageSlug;
  return path.split("/").filter(Boolean).at(-1) ?? path;
}

function FolderIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 6.5A2.5 2.5 0 0 1 5.5 4H9l2 2h7.5A2.5 2.5 0 0 1 21 8.5v8A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5z" />
    </svg>
  );
}

function FileIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" />
      <path d="M14 3v5h5" />
    </svg>
  );
}

export function FileColumnBrowser({
  entries,
  currentFilePath,
  errorMessage,
  isLoading,
  packageSlug,
  onSelectFile,
}: FileColumnBrowserProps) {
  const [activeBrowsePath, setActiveBrowsePath] = useState<string | null>(null);
  const visibleEntries = useMemo(() => visibleSortedEntries(entries), [entries]);
  const normalizedCurrentFilePath = currentFilePath ? normalizePath(currentFilePath) : null;
  const activePath =
    (activeBrowsePath && findEntry(visibleEntries, activeBrowsePath) ? activeBrowsePath : null) ??
    (normalizedCurrentFilePath && findEntry(visibleEntries, normalizedCurrentFilePath) ? normalizedCurrentFilePath : null);
  const directoryChain = getDirectoryChain(visibleEntries, activePath);
  const columns: FileColumn[] = [
    { entries: visibleEntries, label: packageSlug, path: null },
    ...directoryChain
      .map((path) => findEntry(visibleEntries, path))
      .filter((entry): entry is FileEntry => Boolean(entry?.isDirectory))
      .map((entry) => ({
        entries: entry.children ?? [],
        label: getColumnLabel(entry.path, packageSlug),
        path: entry.path,
      })),
  ];

  if (isLoading) {
    return <div className="file-column-state muted">正在读取文件结构...</div>;
  }

  if (errorMessage) {
    return <div className="file-column-state file-column-state-error">{errorMessage}</div>;
  }

  if (visibleEntries.length === 0) {
    return <div className="file-column-state muted">这个 package 里还没有可编辑文件。</div>;
  }

  return (
    <div className="file-column-browser">
      <div className="file-column-scroll" role="tree" aria-label="Package files">
        {columns.map((column, columnIndex) => (
          <section className="file-column" key={column.path ?? "root"}>
            <header className="file-column-header">
              <span>{columnIndex === 0 ? "root" : "folder"}</span>
              <strong title={column.path ?? packageSlug}>{column.label}</strong>
            </header>
            <div className="file-column-rows">
              {column.entries.length === 0 ? (
                <div className="file-column-empty">空目录</div>
              ) : (
                column.entries.map((entry) => {
                  const activeDirectory = entry.isDirectory && directoryChain.includes(entry.path);
                  const activeFile = !entry.isDirectory && normalizedCurrentFilePath === entry.path;
                  const active = activeDirectory || activeFile || activeBrowsePath === entry.path;

                  return (
                    <button
                      className={`file-column-row ${entry.isDirectory ? "is-dir" : "is-file"} ${isSkillFile(entry) ? "is-skill" : ""} ${active ? "is-active" : ""}`}
                      key={entry.path}
                      onClick={() => {
                        setActiveBrowsePath(entry.path);
                        if (!entry.isDirectory) {
                          onSelectFile(entry.path);
                        }
                      }}
                      title={entry.path}
                      type="button"
                    >
                      <span className="file-row-icon">{entry.isDirectory ? <FolderIcon /> : <FileIcon />}</span>
                      <span className="file-row-name">{entry.name}{entry.isDirectory ? "/" : ""}</span>
                      {entry.isDirectory ? <span className="file-row-chevron">›</span> : null}
                    </button>
                  );
                })
              )}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}
