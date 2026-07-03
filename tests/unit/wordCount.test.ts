import { describe, test, expect } from "vitest";
import { docStats } from "../../src/lib/skein/wordCount";

describe("docStats", () => {
  test("empty body is all zeroes", () => {
    expect(docStats("")).toEqual({ words: 0, chars: 0, minutes: 0 });
    expect(docStats("   \n\n  ")).toEqual({ words: 0, chars: 0, minutes: 0 });
  });

  test("counts words across lines", () => {
    const s = docStats("one two three\nfour   five");
    expect(s.words).toBe(5);
    expect(s.minutes).toBe(1);
  });

  test("excludes frontmatter", () => {
    const s = docStats("---\ntitle: x\ntags: [a, b]\n---\n\nreal body words");
    expect(s.words).toBe(3);
  });

  test("excludes CRLF frontmatter", () => {
    const s = docStats("---\r\ntitle: x\r\n---\r\n\r\ntwo words");
    expect(s.words).toBe(2);
  });

  test("counts characters, not bytes", () => {
    expect(docStats("üüü").chars).toBe(3);
  });

  test("reading time scales with length", () => {
    const body = Array(660).fill("word").join(" "); // 660 words / 220 wpm
    expect(docStats(body).minutes).toBe(3);
  });

  test("frontmatter-only body counts as empty", () => {
    expect(docStats("---\ntitle: x\n---\n").words).toBe(0);
  });
});
