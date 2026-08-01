import { describe, it, expect } from "vitest";
import { normalizeVaultRelative } from "../../src/lib/skein/editor/livePreview";

describe("normalizeVaultRelative", () => {
  it("resolves . and redundant separators", () => {
    expect(normalizeVaultRelative("Book/./pic.png")).toBe("Book/pic.png");
    expect(normalizeVaultRelative("Book//pic.png")).toBe("Book/pic.png");
  });

  it("resolves .. that stays inside the vault", () => {
    expect(normalizeVaultRelative("Book/../pic.png")).toBe("pic.png");
    expect(normalizeVaultRelative("A/B/../pic.png")).toBe("A/pic.png");
  });

  it("refuses a climb above the vault root", () => {
    // An imported note carrying ![](../../../../etc/passwd) must not turn
    // into a live asset request for that file.
    expect(normalizeVaultRelative("../pic.png")).toBeNull();
    expect(normalizeVaultRelative("Book/../../etc/passwd")).toBeNull();
  });
});
