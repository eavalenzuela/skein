import { test, expect } from "../fixtures";

async function openPalette(page: import("@playwright/test").Page) {
  await page.goto("/");
  await expect(page.locator(".loading")).toHaveCount(0);
  // Click the Search button rather than relying on Ctrl+K — the global
  // keybinding works in the live app but Playwright's synthetic
  // keypress doesn't reach the svelte:window handler unless something
  // in the page has focus first, which makes the test flaky.
  await page.getByRole("button", { name: /^search$/i }).click();
  await expect(page.getByRole("dialog", { name: /search pages/i })).toBeVisible();
}

test.describe("CommandPalette", () => {
  test("opens via the Search button and closes with Escape", async ({ page }) => {
    await openPalette(page);
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog", { name: /search pages/i })).toHaveCount(0);
  });

  test("page search returns hits from the mock vault", async ({ page }) => {
    await openPalette(page);
    await page.getByRole("textbox", { name: /search pages or commands/i }).fill("alpha");
    await expect(page.locator(".results li").first()).toContainText("Alpha note");
  });

  test("colon prefix switches to command mode and lists built-ins", async ({ page }) => {
    await openPalette(page);
    await page.getByRole("textbox", { name: /search pages or commands/i }).fill(":");
    await expect(page.locator(".results li").first()).toContainText(/open settings/i);
    await expect(
      page.locator(".results li").filter({ hasText: /rebuild search/i }),
    ).toBeVisible();
  });

  test(":settings command opens the Settings modal", async ({ page }) => {
    await openPalette(page);
    await page.getByRole("textbox", { name: /search pages or commands/i }).fill(":settings");
    // Click the command rather than pressing Enter — the same code path runs
    // (selectCommand) but click doesn't bubble Enter through the input.
    await page.locator(".results li").first().click();
    await expect(page.getByRole("dialog", { name: /^settings$/i })).toBeVisible();
  });
});
