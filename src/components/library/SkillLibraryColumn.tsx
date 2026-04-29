import { useMemo, useState } from "react";
import { StatusBadge } from "../common/StatusBadge";
import type { AppBootstrap, PackageStatus, SkillPackage } from "../../types/models";

const STATUS_OPTIONS: Array<{ label: string; value: PackageStatus | "all" }> = [
  { label: "全部", value: "all" },
  { label: "已验证", value: "validated" },
  { label: "待评估", value: "needs_eval" },
  { label: "草稿", value: "draft" },
];

interface SkillLibraryColumnProps {
  bootstrap: AppBootstrap;
  selectedPackageId: string | null;
  onCreate: () => void;
  onSelectPackage: (packageId: string) => void;
}

function getAverageScore(bootstrap: AppBootstrap, item: SkillPackage) {
  const report = bootstrap.evalReports.find((entry) => entry.packageId === item.id);
  if (!report) return null;
  return Math.round(
    ((report.completenessScore + report.clarityScore + report.executabilityScore) / 3) * 100,
  );
}

function matchesQuery(item: SkillPackage, query: string) {
  if (!query) return true;
  const haystack = [
    item.slug,
    item.name,
    item.description,
    item.status,
    ...item.tags,
  ].join(" ").toLowerCase();
  return haystack.includes(query.toLowerCase());
}

export function SkillLibraryColumn({
  bootstrap,
  selectedPackageId,
  onCreate,
  onSelectPackage,
}: SkillLibraryColumnProps) {
  const [query, setQuery] = useState("");
  const [activeStatus, setActiveStatus] = useState<PackageStatus | "all">("all");
  const [activeTag, setActiveTag] = useState<string | null>(null);

  const tags = useMemo(
    () => Array.from(new Set(bootstrap.packages.flatMap((item) => item.tags))).sort((a, b) => a.localeCompare(b)),
    [bootstrap.packages],
  );

  const packages = useMemo(
    () =>
      [...bootstrap.packages]
        .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
        .filter((item) => matchesQuery(item, query))
        .filter((item) => activeStatus === "all" || item.status === activeStatus)
        .filter((item) => !activeTag || item.tags.includes(activeTag)),
    [activeStatus, activeTag, bootstrap.packages, query],
  );

  const validatedCount = bootstrap.packages.filter((item) => item.status === "validated").length;
  const draftCount = bootstrap.packages.filter((item) => item.status === "draft").length;

  return (
    <aside className="skill-library-column" aria-label="Skill Library">
      <div className="library-header">
        <div>
          <h2>Skill Library</h2>
          <p className="muted">{bootstrap.packages.length} 个 skill · {validatedCount} 个已验证 · {draftCount} 个草稿</p>
        </div>
        <button className="library-create-btn" onClick={onCreate} title="生成 Skill" type="button">
          +
        </button>
      </div>

      <label className="library-search">
        <span className="sr-only">搜索 skill</span>
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.35-4.35" />
        </svg>
        <input
          onChange={(event) => setQuery(event.target.value)}
          placeholder="搜索 slug、名称、标签"
          value={query}
        />
      </label>

      <div className="library-filter-row" aria-label="状态筛选">
        {STATUS_OPTIONS.map((option) => (
          <button
            className={`library-chip ${activeStatus === option.value ? "is-active" : ""}`}
            key={option.value}
            onClick={() => setActiveStatus(option.value)}
            type="button"
          >
            {option.label}
          </button>
        ))}
      </div>

      {tags.length > 0 ? (
        <div className="library-tag-strip" aria-label="标签筛选">
          <button
            className={`library-tag ${activeTag === null ? "is-active" : ""}`}
            onClick={() => setActiveTag(null)}
            type="button"
          >
            全部标签
          </button>
          {tags.map((tag) => (
            <button
              className={`library-tag ${activeTag === tag ? "is-active" : ""}`}
              key={tag}
              onClick={() => setActiveTag(tag)}
              type="button"
            >
              {tag}
            </button>
          ))}
        </div>
      ) : null}

      <div className="library-list">
        {packages.length === 0 ? (
          <div className="library-empty">
            <strong>没有匹配的 skill</strong>
            <span>换个关键词，或清掉筛选条件。</span>
          </div>
        ) : (
          packages.map((item) => {
            const score = getAverageScore(bootstrap, item);
            return (
              <button
                className={`skill-library-row ${selectedPackageId === item.id ? "is-active" : ""}`}
                key={item.id}
                onClick={() => onSelectPackage(item.id)}
                type="button"
              >
                <span className="skill-row-main">
                  <span className="skill-row-title">
                    <span className="skill-row-name">{item.name}</span>
                    <StatusBadge status={item.status} />
                  </span>
                  <span className="skill-row-slug">{item.slug}</span>
                  <span className="skill-row-desc">{item.description}</span>
                </span>
                <span className="skill-row-foot">
                  <span>v{item.currentVersion}</span>
                  {score !== null ? <span>{score}</span> : <span>未评估</span>}
                </span>
              </button>
            );
          })
        )}
      </div>
    </aside>
  );
}
