import { describe, test, expect } from "vitest";
import { spineHue, spineHeight, spineShade, spineWidth } from "../../src/lib/skein/spineHash";

describe("spineHash", () => {
  test("hue is one of the palette entries", () => {
    const PALETTE = [28, 145, 220, 350, 60, 195, 25, 130, 280, 95, 175];
    expect(PALETTE).toContain(spineHue("Research"));
    expect(PALETTE).toContain(spineHue(""));
    expect(PALETTE).toContain(spineHue("a-very-long-book-name"));
  });

  test("hue is deterministic", () => {
    expect(spineHue("Research")).toBe(spineHue("Research"));
    expect(spineHue("Daily")).not.toBe(spineHue("Daily extra"));
  });

  test("height is between 74 and 84", () => {
    for (const name of ["A", "B", "Recipes", "Daily Notes", "Project Ideas"]) {
      const h = spineHeight(name);
      expect(h).toBeGreaterThanOrEqual(74);
      expect(h).toBeLessThanOrEqual(84);
    }
  });

  test("shade is between 0.28 and 0.34", () => {
    for (const name of ["A", "B", "Research", "X"]) {
      const s = spineShade(name);
      expect(s).toBeGreaterThanOrEqual(0.28);
      // Floating-point arithmetic: 0.28 + 6/100 = 0.34, but allow a hair.
      expect(s).toBeLessThanOrEqual(0.341);
    }
  });

  test("width is base 28 for short names and grows for long ones", () => {
    expect(spineWidth("A")).toBe(28);
    expect(spineWidth("Research")).toBe(28); // 8 chars
    expect(spineWidth("0123456789")).toBe(28); // 10 chars (cap)
    expect(spineWidth("01234567890")).toBe(38); // 11 chars → +10
    expect(spineWidth("a really really really really really long book name")).toBeGreaterThan(28);
  });
});
