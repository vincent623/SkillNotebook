import { startTransition, useDeferredValue, useState } from "react";
import { SearchBar } from "../common/SearchBar";
import { PackageList } from "../package/PackageList";
import { usePackageStore } from "../../stores/package-store";
import { useWorkspaceStore } from "../../stores/workspace-store";
import type { SkillPackage, Workspace } from "../../types/models";

interface SidebarProps {
  workspace: Workspace;
  packages: SkillPackage[];
  selectedPackageId: string | null;
  onSelectPackage: (packageId: string) => void;
}

export function Sidebar({
  workspace,
  packages,
  selectedPackageId,
  onSelectPackage,
}: SidebarProps) {
  const activeTab = usePackageStore((state) => state.activeTab);
  const setActiveTab = usePackageStore((state) => state.setActiveTab);
  const createComposerOpen = useWorkspaceStore((state) => state.createComposerOpen);
  const createPrompt = useWorkspaceStore((state) => state.createPrompt);
  const createContext = useWorkspaceStore((state) => state.createContext);
  const createStatus = useWorkspaceStore((state) => state.createStatus);
  const createError = useWorkspaceStore((state) => state.createError);
  const lastCreateResult = useWorkspaceStore((state) => state.lastCreateResult);
  const toggleCreateComposer = useWorkspaceStore((state) => state.toggleCreateComposer);
  const setCreatePrompt = useWorkspaceStore((state) => state.setCreatePrompt);
  const setCreateContext = useWorkspaceStore((state) => state.setCreateContext);
  const createPackage = useWorkspaceStore((state) => state.createPackage);
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);

  const filteredPackages = packages.filter((item) => {
    const haystack = `${item.name} ${item.description} ${item.tags.join(" ")}`.toLowerCase();
    return haystack.includes(deferredQuery.trim().toLowerCase());
  });

  const isSubmitting = createStatus === "submitting";
  const generatorLabel =
    lastCreateResult?.generatorUsed === "skill_create_cli"
      ? "skill-create"
      : lastCreateResult?.generatorUsed === "claude_cli"
        ? "Claude CLI"
        : "Template fallback";

  return (
    <aside className="workspace-sidebar">
      <div className="sidebar-top">
        <p className="eyebrow">Workspace</p>
        <h2>{workspace.name}</h2>
        <p className="muted mono-text">{workspace.rootPath}</p>
      </div>

      <div className="button-row">
        <button
          className="button-primary"
          onClick={() => {
            toggleCreateComposer(!createComposerOpen);
          }}
          type="button"
        >
          {createComposerOpen ? "Close Composer" : "New Package"}
        </button>
        <button
          className="button-secondary"
          disabled
          type="button"
        >
          Import Soon
        </button>
      </div>

      {createComposerOpen ? (
        <div className="content-card compact composer-card">
          <div className="section-head">
            <h3>New package</h3>
            <span className="section-meta">Natural language draft</span>
          </div>

          <label className="field-stack">
            <span className="search-label">What should this skill do?</span>
            <textarea
              className="form-textarea"
              onChange={(event) => {
                setCreatePrompt(event.target.value);
              }}
              placeholder="Example: Turn customer interview notes into tagged insights, tensions, and next-step recommendations."
              rows={5}
              value={createPrompt}
            />
          </label>

          <label className="field-stack">
            <span className="search-label">Extra context</span>
            <textarea
              className="form-textarea form-textarea-secondary"
              onChange={(event) => {
                setCreateContext(event.target.value);
              }}
              placeholder="Optional constraints, preferred output shape, source formats, or trigger hints."
              rows={4}
              value={createContext}
            />
          </label>

          {createError ? <div className="inline-banner inline-banner-error">{createError}</div> : null}

          <div className="button-row">
            <button
              className="button-primary"
              disabled={isSubmitting}
              onClick={async () => {
                const result = await createPackage();
                if (!result) {
                  return;
                }

                startTransition(() => {
                  if (activeTab !== "overview") {
                    setActiveTab("overview");
                  }
                });
              }}
              type="button"
            >
              {isSubmitting ? "Generating Draft..." : "Generate Draft"}
            </button>
            <button
              className="button-secondary"
              disabled={isSubmitting}
              onClick={() => {
                toggleCreateComposer(false);
              }}
              type="button"
            >
              Cancel
            </button>
          </div>
        </div>
      ) : null}

      {lastCreateResult ? (
        <div className="content-card compact composer-result">
          <div className="section-head">
            <h3>Last draft</h3>
            <span className="section-meta">{generatorLabel}</span>
          </div>
          <p className="body-copy">
            {lastCreateResult.name} is ready and selected in the workspace.
          </p>
          <p className="muted">{lastCreateResult.generationSummary}</p>
          <p className="muted">{lastCreateResult.validationSummary}</p>
          <div className="meta-pill muted-pill">{lastCreateResult.slug}</div>
        </div>
      ) : null}

      <SearchBar
        value={query}
        onChange={setQuery}
      />

      <div className="sidebar-section">
        <div className="section-head">
          <h3>Library</h3>
          <span className="section-meta">{filteredPackages.length} items</span>
        </div>
        <PackageList
          packages={filteredPackages}
          selectedPackageId={selectedPackageId}
          onSelect={onSelectPackage}
        />
      </div>
    </aside>
  );
}
