// Extract the frontmatter `tags:` field from a markdown body. Supports
// both the inline (`tags: [a, b]`) and block (`tags:\n  - a`) YAML forms.
// Used by TagChips to filter already-applied tags out of the suggestion
// list. Kept frontmatter-only — we don't try to parse the rest of the YAML.

export function parseFrontmatterTags(body: string): Set<string> {
  const m = body.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!m) return new Set();
  const fm = m[1];
  const inline = fm.match(/^tags:\s*\[([^\]]*)\]/m);
  if (inline) {
    return new Set(
      inline[1]
        .split(",")
        .map((s) => s.trim().replace(/^["']|["']$/g, ""))
        .filter(Boolean),
    );
  }
  const block = fm.match(/^tags:\s*\n((?:\s*-\s*.+\n?)+)/m);
  if (block) {
    return new Set(
      block[1]
        .split("\n")
        .map((l) =>
          l
            .replace(/^\s*-\s*/, "")
            .trim()
            .replace(/^["']|["']$/g, ""),
        )
        .filter(Boolean),
    );
  }
  return new Set();
}
