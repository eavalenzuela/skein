import { describe, test, expect } from "vitest";
import { parseFrontmatterTags } from "../../src/lib/skein/frontmatter";

describe("parseFrontmatterTags", () => {
  test("returns empty set when no frontmatter", () => {
    expect(parseFrontmatterTags("just body, no fm")).toEqual(new Set());
  });

  test("inline form: tags: [a, b]", () => {
    const body = "---\ntitle: x\ntags: [garden, plants]\n---\n\nbody";
    expect(parseFrontmatterTags(body)).toEqual(new Set(["garden", "plants"]));
  });

  test("inline form strips quotes", () => {
    const body = `---\ntags: ["a", 'b']\n---\n`;
    expect(parseFrontmatterTags(body)).toEqual(new Set(["a", "b"]));
  });

  test("block form", () => {
    const body = `---\ntitle: x\ntags:\n  - alpha\n  - beta\n  - gamma\n---\n\nbody`;
    expect(parseFrontmatterTags(body)).toEqual(new Set(["alpha", "beta", "gamma"]));
  });

  test("ignores tags outside frontmatter", () => {
    const body = "no fm here\n\ntags: [should, not, count]";
    expect(parseFrontmatterTags(body)).toEqual(new Set());
  });

  test("handles CRLF line endings", () => {
    const body = "---\r\ntitle: x\r\ntags: [a, b]\r\n---\r\n\r\nbody";
    expect(parseFrontmatterTags(body)).toEqual(new Set(["a", "b"]));
  });

  test("empty inline list yields empty set", () => {
    const body = "---\ntags: []\n---\n";
    expect(parseFrontmatterTags(body)).toEqual(new Set());
  });
});
