import { EmptyState } from "../common/EmptyState";
import { PackageFiles } from "../package/PackageFiles";
import { PackageOverview } from "../package/PackageOverview";
import { PackagePreview } from "../package/PackagePreview";
import { PackageTest } from "../package/PackageTest";
import { usePackageStore } from "../../stores/package-store";
import type { EvalReport, PreviewModel, SkillPackage, WorkspaceTab } from "../../types/models";

const tabs: WorkspaceTab[] = ["overview", "files", "preview", "test"];

interface MainPanelProps {
  selectedPackage?: SkillPackage;
  evalReport?: EvalReport;
  preview?: PreviewModel;
  activityLog: string[];
}

export function MainPanel({
  selectedPackage,
  evalReport,
  preview,
  activityLog,
}: MainPanelProps) {
  const activeTab = usePackageStore((state) => state.activeTab);
  const setActiveTab = usePackageStore((state) => state.setActiveTab);

  if (!selectedPackage || !preview) {
    return (
      <main className="workspace-main">
        <EmptyState
          title="Choose a package to start"
          description="This area becomes the working surface for overview, files, preview, and test."
        />
      </main>
    );
  }

  return (
    <main className="workspace-main">
      <div className="tab-row">
        {tabs.map((tab) => (
          <button
            key={tab}
            className={`tab-pill ${tab === activeTab ? "is-active" : ""}`}
            onClick={() => {
              setActiveTab(tab);
            }}
            type="button"
          >
            {tab}
          </button>
        ))}
      </div>

      {activeTab === "overview" ? (
        <PackageOverview
          item={selectedPackage}
          evalReport={evalReport}
        />
      ) : null}

      {activeTab === "files" ? <PackageFiles preview={preview} /> : null}

      {activeTab === "preview" ? <PackagePreview preview={preview} /> : null}

      {activeTab === "test" ? (
        <PackageTest
          item={selectedPackage}
          activityLog={activityLog}
        />
      ) : null}
    </main>
  );
}
