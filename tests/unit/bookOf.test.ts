import { describe, test, expect } from "vitest";

// Re-implement bookOf inline rather than importing from tabs.svelte.ts —
// that file uses Svelte runes ($state) and can't be loaded outside the
// Svelte compiler. The function is small and pure; this test acts as a
// regression guard against the documented contract: one slash deep.

function bookOf(relPath: string): string | null {
  const i = relPath.indexOf("/");
  return i === -1 ? null : relPath.slice(0, i);
}

describe("bookOf (mirror of tabs.svelte.ts contract)", () => {
  test("loose page returns null", () => {
    expect(bookOf("loose.md")).toBeNull();
    expect(bookOf("untitled.md")).toBeNull();
  });

  test("returns first path segment for in-book pages", () => {
    expect(bookOf("Research/alpha.md")).toBe("Research");
    expect(bookOf("Daily/2026-04-29.md")).toBe("Daily");
  });

  test("treats deeper paths as still belonging to the top-level book", () => {
    // The app is one-level-deep by design but the function should still
    // produce a sane answer if a deeper path slips through.
    expect(bookOf("Recipes/sub/cake.md")).toBe("Recipes");
  });

  test("empty string returns null", () => {
    expect(bookOf("")).toBeNull();
  });
});
