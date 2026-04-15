import type { PackageVersion } from "../../types/models";

interface VersionDiffModalProps {
  versions: PackageVersion[];
}

export function VersionDiffModal({ versions }: VersionDiffModalProps) {
  const latest = versions[0];
  const previous = versions[1];

  return (
    <div className="content-card">
      <div className="section-head">
        <h3>Diff glance</h3>
        <span className="section-meta">Placeholder surface</span>
      </div>
      {latest && previous ? (
        <p className="muted">
          Compare <strong>v{previous.versionNumber}</strong> to <strong>v{latest.versionNumber}</strong>
          {" "}once the diff service is wired into Rust.
        </p>
      ) : (
        <p className="muted">A richer diff viewer lands after version snapshots and restore flow exist.</p>
      )}
    </div>
  );
}
