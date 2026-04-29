import type { ReactNode } from "react";
import { parseFrontmatter } from "./markdown-utils";

interface MarkdownPreviewProps {
  content: string;
}

function renderInline(text: string, keyPrefix: string): ReactNode[] {
  return text
    .split(/(`[^`]+`|\*\*[^*]+?\*\*)/g)
    .filter((part) => part.length > 0)
    .map((part, index) => {
      const key = `${keyPrefix}-${index}`;
      if (part.startsWith("`") && part.endsWith("`")) {
        return (
          <code className="markdown-inline-code" key={key}>
            {part.slice(1, -1)}
          </code>
        );
      }
      if (part.startsWith("**") && part.endsWith("**")) {
        return <strong key={key}>{part.slice(2, -2)}</strong>;
      }
      return part;
    });
}

function isBlockStart(line: string) {
  return (
    /^```/.test(line) ||
    /^(#{1,4})\s+/.test(line) ||
    /^\s*[-*]\s+/.test(line) ||
    /^\s*\d+\.\s+/.test(line) ||
    /^>\s?/.test(line) ||
    /^[-*_]{3,}\s*$/.test(line.trim())
  );
}

function renderHeading(level: number, text: string, key: string) {
  const content = renderInline(text, key);
  if (level === 1) return <h1 key={key}>{content}</h1>;
  if (level === 2) return <h2 key={key}>{content}</h2>;
  if (level === 3) return <h3 key={key}>{content}</h3>;
  return <h4 key={key}>{content}</h4>;
}

function renderBlocks(markdown: string): ReactNode[] {
  const lines = markdown.replace(/\r\n/g, "\n").split("\n");
  const nodes: ReactNode[] = [];
  let index = 0;
  let blockIndex = 0;

  while (index < lines.length) {
    const line = lines[index];
    const trimmed = line.trim();

    if (!trimmed) {
      index += 1;
      continue;
    }

    const fence = line.match(/^```([A-Za-z0-9_-]+)?\s*$/);
    if (fence) {
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !/^```\s*$/.test(lines[index])) {
        codeLines.push(lines[index]);
        index += 1;
      }
      if (index < lines.length) index += 1;
      nodes.push(
        <pre className="markdown-code-block" key={`code-${blockIndex}`}>
          <code data-language={fence[1] ?? undefined}>{codeLines.join("\n")}</code>
        </pre>,
      );
      blockIndex += 1;
      continue;
    }

    const heading = line.match(/^(#{1,4})\s+(.+)$/);
    if (heading) {
      nodes.push(renderHeading(heading[1].length, heading[2], `heading-${blockIndex}`));
      blockIndex += 1;
      index += 1;
      continue;
    }

    if (/^[-*_]{3,}\s*$/.test(trimmed)) {
      nodes.push(<hr key={`hr-${blockIndex}`} />);
      blockIndex += 1;
      index += 1;
      continue;
    }

    if (/^\s*[-*]\s+/.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\s*[-*]\s+/.test(lines[index])) {
        items.push(lines[index].replace(/^\s*[-*]\s+/, ""));
        index += 1;
      }
      nodes.push(
        <ul key={`ul-${blockIndex}`}>
          {items.map((item, itemIndex) => (
            <li key={`${blockIndex}-${itemIndex}`}>{renderInline(item, `ul-${blockIndex}-${itemIndex}`)}</li>
          ))}
        </ul>,
      );
      blockIndex += 1;
      continue;
    }

    if (/^\s*\d+\.\s+/.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\s*\d+\.\s+/.test(lines[index])) {
        items.push(lines[index].replace(/^\s*\d+\.\s+/, ""));
        index += 1;
      }
      nodes.push(
        <ol key={`ol-${blockIndex}`}>
          {items.map((item, itemIndex) => (
            <li key={`${blockIndex}-${itemIndex}`}>{renderInline(item, `ol-${blockIndex}-${itemIndex}`)}</li>
          ))}
        </ol>,
      );
      blockIndex += 1;
      continue;
    }

    if (/^>\s?/.test(line)) {
      const quoteLines: string[] = [];
      while (index < lines.length && /^>\s?/.test(lines[index])) {
        quoteLines.push(lines[index].replace(/^>\s?/, ""));
        index += 1;
      }
      nodes.push(
        <blockquote key={`quote-${blockIndex}`}>
          {renderInline(quoteLines.join(" "), `quote-${blockIndex}`)}
        </blockquote>,
      );
      blockIndex += 1;
      continue;
    }

    const paragraphLines: string[] = [];
    while (index < lines.length && lines[index].trim() && !isBlockStart(lines[index])) {
      paragraphLines.push(lines[index].trim());
      index += 1;
    }

    nodes.push(
      <p key={`p-${blockIndex}`}>
        {renderInline(paragraphLines.join(" "), `p-${blockIndex}`)}
      </p>,
    );
    blockIndex += 1;
  }

  return nodes;
}

export function MarkdownPreview({ content }: MarkdownPreviewProps) {
  const parsed = parseFrontmatter(content);
  const body = parsed.hasFrontmatter ? parsed.body : content;
  const nodes = renderBlocks(body);

  return (
    <article className="markdown-preview">
      {nodes.length > 0 ? nodes : <p className="muted">这个 Markdown 文件暂时没有正文内容。</p>}
    </article>
  );
}
