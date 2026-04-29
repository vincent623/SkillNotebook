export interface ParsedFrontmatter {
  body: string;
  hasFrontmatter: boolean;
  metadata: Record<string, string>;
}

function stripQuotes(value: string) {
  if (
    (value.startsWith("\"") && value.endsWith("\"")) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1);
  }
  return value;
}

export function parseFrontmatter(content: string): ParsedFrontmatter {
  const normalized = content.replace(/\r\n/g, "\n");
  if (!normalized.startsWith("---\n")) {
    return { body: content, hasFrontmatter: false, metadata: {} };
  }

  const lines = normalized.split("\n");
  const closingIndex = lines.findIndex((line, index) => index > 0 && /^---\s*$/.test(line));
  if (closingIndex < 0) {
    return { body: content, hasFrontmatter: false, metadata: {} };
  }

  const metadata: Record<string, string> = {};
  for (const line of lines.slice(1, closingIndex)) {
    const match = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (match) {
      metadata[match[1]] = stripQuotes(match[2].trim());
    }
  }

  return {
    body: lines.slice(closingIndex + 1).join("\n").trimStart(),
    hasFrontmatter: true,
    metadata,
  };
}
