import { describe, test, expect } from "vitest";
import { escapeHtml, sanitizeSnippet } from "../../src/lib/skein/sanitize";

describe("escapeHtml", () => {
  test("escapes the dangerous four", () => {
    expect(escapeHtml(`<a href="x">&</a>`)).toBe(
      "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;",
    );
  });

  test("leaves plain text alone", () => {
    expect(escapeHtml("just words")).toBe("just words");
  });
});

describe("sanitizeSnippet", () => {
  test("preserves FTS mark highlights", () => {
    expect(sanitizeSnippet("a <mark>hit</mark> here")).toBe(
      "a <mark>hit</mark> here",
    );
  });

  test("neutralizes markup living in note content", () => {
    const out = sanitizeSnippet(`<img src=x onerror=alert(1)> <mark>x</mark>`);
    expect(out).toContain("&lt;img src=x onerror=alert(1)&gt;");
    expect(out).toContain("<mark>x</mark>");
    expect(out).not.toContain("<img");
  });

  test("script tags cannot survive", () => {
    const out = sanitizeSnippet("<script>alert(1)</script>");
    expect(out).toBe("&lt;script&gt;alert(1)&lt;/script&gt;");
  });

  test("marked-up text inside a highlight stays escaped", () => {
    const out = sanitizeSnippet("<mark><b>bold</b></mark>");
    expect(out).toBe("<mark>&lt;b&gt;bold&lt;/b&gt;</mark>");
  });
});
