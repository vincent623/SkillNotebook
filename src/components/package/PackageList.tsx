import { PackageItem } from "./PackageItem";
import type { SkillPackage } from "../../types/models";

interface PackageListProps {
  packages: SkillPackage[];
  selectedPackageId: string | null;
  onSelect: (packageId: string) => void;
}

export function PackageList({ packages, selectedPackageId, onSelect }: PackageListProps) {
  return (
    <div className="package-list">
      {packages.map((item) => (
        <PackageItem
          key={item.id}
          item={item}
          isActive={selectedPackageId === item.id}
          onSelect={onSelect}
        />
      ))}
    </div>
  );
}
