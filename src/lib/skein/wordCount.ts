// Word/character stats for the editor footer. Frontmatter is metadata,
// not prose, so it's excluded before counting.

export interface DocStats {
  words: number;
  chars: number;
  /** Estimated reading time in whole minutes, at ~220 wpm; minimum 1 when
   * there is any content at all. */
  minutes: number;
}

const READING_WPM = 220;

function stripFrontmatter(body: string): string {
  const m = body.match(/^---\r?\n[\s\S]*?\r?\n---(?:\r?\n|$)/);
  return m ? body.slice(m[0].length) : body;
}

export function docStats(body: string): DocStats {
  const text = stripFrontmatter(body).trim();
  if (text === "") return { words: 0, chars: 0, minutes: 0 };
  const words = text.split(/\s+/).filter(Boolean).length;
  const chars = [...text].length;
  const minutes = Math.max(1, Math.round(words / READING_WPM));
  return { words, chars, minutes };
}
