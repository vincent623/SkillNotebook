import { useMemo, useState } from "react";
import { StatusBadge } from "../common/StatusBadge";
import type { AppBootstrap, EvalReport, SkillPackage } from "../../types/models";

interface SkillLibraryColumnProps {
  bootstrap: AppBootstrap;
  selectedPackageId: string | null;
  onCreate: () => void;
  onSelectPackage: (packageId: string) => void;
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

function formatShortDate(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString("zh-CN", { month: "short", day: "numeric" });
}

function SearchIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" />
    </svg>
  );
}

function XIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 4l8 8M12 4l-8 8" />
    </svg>
  );
}

export function SkillLibraryColumn({
  bootstrap,
  selectedPackageId,
  onCreate,
  onSelectPackage,
}: SkillLibraryColumnProps) {
  const [query, setQuery] = useState("");
  const [activeTag, setActiveTag] = useState<string | null>(null);

  const tags = useMemo(
    () => {
      const counts = new Map<string, number>();
      bootstrap.packages.forEach((item) => {
        item.tags.forEach((tag) => counts.set(tag, (counts.get(tag) ?? 0) + 1));
      });
      return Array.from(counts.entries()).sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]));
    },
    [bootstrap.packages],
  );

  const packages = useMemo(
    () =>
      [...bootstrap.packages]
        .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
        .filter((item) => matchesQuery(item, query))
        .filter((item) => !activeTag || item.tags.includes(activeTag)),
    [activeTag, bootstrap.packages, query],
  );

  return (
    <aside className="skill-library-column" aria-label="Skill Library">
      <div className="library-header">
        <h2>Skills · {bootstrap.packages.length}</h2>
        <button className="library-create-btn" onClick={onCreate} title="生成 Skill" type="button">
          +
        </button>
      </div>

      <label className="library-search">
        <span className="sr-only">搜索 skill</span>
        <SearchIcon />
        <input
          onChange={(event) => setQuery(event.target.value)}
          placeholder="搜索名称、描述、标签"
          value={query}
        />
        {query ? (
          <button className="library-clear-btn" onClick={() => setQuery("")} type="button">
            <XIcon />
          </button>
        ) : null}
      </label>

      {tags.length > 0 ? (
        <div className="library-tag-strip" aria-label="标签筛选">
          <button
            className={`library-tag ${activeTag === null ? "is-active" : ""}`}
            onClick={() => setActiveTag(null)}
            type="button"
          >
            全部 <span>{bootstrap.packages.length}</span>
          </button>
          {tags.slice(0, 10).map(([tag, count]) => (
            <button
              className={`library-tag ${activeTag === tag ? "is-active" : ""}`}
              key={tag}
              onClick={() => setActiveTag(tag)}
              type="button"
            >
              {tag} <span>{count}</span>
            </button>
          ))}
        </div>
      ) : null}

      <div className="library-list">
        {packages.length === 0 ? (
          <div className="library-empty">
            没有匹配的 skill
          </div>
        ) : (
          packages.map((item) => (
            <SkillRow
              active={selectedPackageId === item.id}
              evalReport={bootstrap.evalReports.find((report) => report.packageId === item.id)}
              item={item}
              key={item.id}
              onSelectPackage={onSelectPackage}
            />
          ))
        )}
      </div>
    </aside>
  );
}

function SkillRow({
  active,
  evalReport,
  item,
  onSelectPackage,
}: {
  active: boolean;
  evalReport?: EvalReport;
  item: SkillPackage;
  onSelectPackage: (packageId: string) => void;
}) {
  const score = evalReport
    ? Math.round(((evalReport.completenessScore + evalReport.clarityScore + evalReport.executabilityScore) / 3) * 100)
    : null;

  return (
    <button
      className={`skill-library-row ${active ? "is-active" : ""}`}
      onClick={() => onSelectPackage(item.id)}
      type="button"
    >
      <span className="skill-row-title">
        <code className="skill-row-name">{item.slug}</code>
        <span className="skill-row-version">v{item.currentVersion}</span>
      </span>
      <span className="skill-row-desc">{item.description}</span>
      <span className="skill-row-quality">
        <StatusBadge status={item.status} />
        <span className={`skill-row-score ${score == null ? "is-empty" : ""}`}>
          {score == null ? "未评估" : `${score}`}
        </span>
      </span>
      <span className="skill-row-foot">
        <span>{formatShortDate(item.updatedAt)}</span>
        {item.tags.slice(0, 2).map((tag) => (
          <code className="skill-row-tag" key={tag}>{tag}</code>
        ))}
      </span>
    </button>
  );
}
