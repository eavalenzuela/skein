import { test, expect } from "../fixtures";

async function haveConversation(page: import("@playwright/test").Page) {
  // Runs after the fixture's mock install, so the key is present before
  // the sidebar reads it on mount.
  await page.addInitScript(() => {
    const w = window as unknown as { __SKEIN_MOCK__: { state: { secrets: Set<string> } } };
    w.__SKEIN_MOCK__.state.secrets.add("anthropic_api_key");
  });
  await page.goto("/");
  await expect(page.locator(".loading")).toHaveCount(0);

  const box = page.locator(".sk-input textarea");
  await box.fill("What is in my vault?");
  await box.press("Enter");
  await expect(page.locator(".sk-msg.asst")).toContainText("Mock response");
}

test.describe("Chat conversation actions", () => {
  test("new conversation clears the transcript", async ({ page }) => {
    await haveConversation(page);
    await expect(page.locator(".sk-msg")).toHaveCount(2);

    await page.getByRole("button", { name: /new conversation/i }).click();
    await expect(page.locator(".sk-msg")).toHaveCount(0);
    // The action only exists while there is something to clear.
    await expect(page.getByRole("button", { name: /new conversation/i })).toHaveCount(0);
  });

  test("saving the conversation writes a page and opens it", async ({ page }) => {
    await haveConversation(page);
    await page.getByRole("button", { name: /save conversation to a note/i }).click();

    await expect(page.locator(".toast")).toContainText(/saved the conversation/i);
    const tab = page.locator(".sk-tab").filter({ hasText: /Chat \d/ });
    await expect(tab).toBeVisible();

    const bodies = await page.evaluate(() => {
      const w = window as unknown as {
        __SKEIN_MOCK__: { state: { pageBodies: Map<string, string> } };
      };
      return [...w.__SKEIN_MOCK__.state.pageBodies.entries()];
    });
    const saved = bodies.find(([rel]) => rel.startsWith("Chat "));
    expect(saved, "a Chat page should have been written").toBeTruthy();
    expect(saved![1]).toContain("**You:** What is in my vault?");
    expect(saved![1]).toContain("Mock response");
    // Frontmatter so it indexes and tags like any other page.
    expect(saved![1]).toMatch(/^---\ntitle: Chat /);
  });
});
