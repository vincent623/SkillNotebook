import { parseFrontmatter } from "./markdown-utils";

const FIELD_LABELS: Record<string, string> = {
  author: "作者",
  created: "创建",
  id: "ID",
  status: "状态",
  tags: "标签",
  updated: "更新",
  version: "版本",
};

function getFallbackName(filePath?: string | null) {
  if (!filePath) return "Skill document";
  return filePath.split("/").filter(Boolean).at(-1) ?? filePath;
}

function getMarkdownSummary(content: string) {
  const lines = content.replace(/\r\n/g, "\n").split("\n");
  const title = lines
    .map((line) => line.match(/^#\s+(.+)$/)?.[1]?.trim())
    .find((line): line is string => Boolean(line));
  const description = lines
    .map((line) => line.trim())
    .find((line) => line.length > 0 && !line.startsWith("#") && !line.startsWith("-"));

  return { description, title };
}

interface FrontmatterCardProps {
  content: string;
  filePath?: string | null;
}

export function FrontmatterCard({ content, filePath }: FrontmatterCardProps) {
  const parsed = parseFrontmatter(content);
  const fileName = getFallbackName(filePath);
  const isSkillMarkdown = fileName.toLowerCase() === "skill.md";
  if (!parsed.hasFrontmatter && !isSkillMarkdown) return null;

  const markdownSummary = getMarkdownSummary(parsed.body);
  const title = parsed.metadata.name || parsed.metadata.title || markdownSummary.title || fileName;
  const description = parsed.metadata.description || parsed.metadata.summary || markdownSummary.description;
  const detailKeys = Object.keys(parsed.metadata)
    .filter((key) => !["name", "title", "description", "summary"].includes(key))
    .slice(0, 6);

  return (
    <section className="frontmatter-card" aria-label="文档元信息">
      <div className="frontmatter-heading">
        <span className="frontmatter-eyebrow">
          {parsed.hasFrontmatter ? (isSkillMarkdown ? "SKILL.md 元信息" : "Markdown 元信息") : "SKILL.md 摘要"}
        </span>
        <h2 className="frontmatter-title">{title}</h2>
        {description ? <p className="frontmatter-description">{description}</p> : null}
      </div>
      {detailKeys.length > 0 ? (
        <dl className="frontmatter-details">
          {detailKeys.map((key) => (
            <div className="frontmatter-detail" key={key}>
              <dt>{FIELD_LABELS[key] ?? key}</dt>
              <dd>{parsed.metadata[key]}</dd>
            </div>
          ))}
        </dl>
      ) : null}
    </section>
  );
}
