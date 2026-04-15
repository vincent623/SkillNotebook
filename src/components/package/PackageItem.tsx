import { StatusBadge } from "../common/StatusBadge";
import type { SkillPackage } from "../../types/models";

interface PackageItemProps {
  item: SkillPackage;
  isActive: boolean;
  onSelect: (packageId: string) => void;
}

export function PackageItem({ item, isActive, onSelect }: PackageItemProps) {
  return (
    <button
      className={`package-item ${isActive ? "is-active" : ""}`}
      onClick={() => {
        onSelect(item.id);
      }}
      type="button"
    >
      <div className="package-item-head">
        <div>
          <p className="package-title">{item.name}</p>
          <p className="package-slug">{item.slug}</p>
        </div>
        <StatusBadge status={item.status} />
      </div>
      <p className="package-description">{item.description}</p>
      <div className="package-meta-row">
        <span className="meta-pill">v{item.currentVersion}</span>
        {item.tags.slice(0, 3).map((tag) => (
          <span
            key={tag}
            className="meta-pill muted-pill"
          >
            {tag}
          </span>
        ))}
      </div>
    </button>
  );
}
