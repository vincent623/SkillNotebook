import { useState } from "react";
import { useWorkspaceStore } from "../../stores/workspace-store";
import type { EvalReport, PackageVersion } from "../../types/models";

interface VersionListProps {
  packageId: string;
  versions: PackageVersion[];
  evalReport?: EvalReport;
}

export function VersionList({ packageId, versions, evalReport }: VersionListProps) {
  const [note, setNote] = useState("");
  const saveVersion = useWorkspaceStore((state) => state.saveVersion);
  const versionSaveStatus = useWorkspaceStore((state) => state.versionSaveStatus);
  const versionSaveError = useWorkspaceStore((state) => state.versionSaveError);
  const lastVersionSavedPackageId = useWorkspaceStore((state) => state.lastVersionSavedPackageId);
  const lastVersionSavedAt = useWorkspaceStore((state) => state.lastVersionSavedAt);

  const isSaving = versionSaveStatus === "submitting" && lastVersionSavedPackageId === packageId;
  const lastSaveError =
    lastVersionSavedPackageId === packageId && versionSaveStatus === "error"
      ? versionSaveError
      : null;
  const lastSaveSuccessAt =
    lastVersionSavedPackageId === packageId && versionSaveStatus === "success"
      ? lastVersionSavedAt
      : null;

  return (
    <div className="content-card">
      <div className="section-head">
        <h3>Versions</h3>
        <span className="section-meta">Formal history</span>
      </div>
      <div className="button-row">
        <input
          className="search-input"
          onChange={(event) => setNote(event.target.value)}
          placeholder={evalReport ? "Optional release note" : "Run an eval before saving a version"}
          value={note}
        />
        <button
          className="button-primary"
          disabled={!evalReport || isSaving}
          onClick={async () => {
            const ok = await saveVersion(packageId, note.trim() || null);
            if (ok) {
              setNote("");
            }
          }}
          type="button"
        >
          {isSaving ? "Saving..." : "Save version"}
        </button>
      </div>
      {lastSaveSuccessAt ? (
        <div className="inline-banner inline-banner-success">
          Saved formal version on {lastSaveSuccessAt.slice(0, 19).replace("T", " ")} UTC.
        </div>
      ) : null}
      {lastSaveError ? (
        <div className="inline-banner inline-banner-error">{lastSaveError}</div>
      ) : null}
      <div className="version-list">
        {versions.length === 0 ? <p className="muted">No formal versions yet.</p> : null}
        {versions.map((item) => (
          <div
            key={item.id}
            className="version-row"
          >
            <div>
              <p className="version-title">
                v{item.versionNumber} {item.isPinned ? "• pinned" : ""}
              </p>
              <p className="muted">{item.note ?? "No release note yet."}</p>
            </div>
            <span className="section-meta">{item.createdAt.slice(0, 10)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
