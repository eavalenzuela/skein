import { test, expect } from "../fixtures";

async function openSettings(page: import("@playwright/test").Page) {
  await page.goto("/");
  await expect(page.locator(".loading")).toHaveCount(0);
  await page.getByRole("button", { name: /^settings/i }).first().click();
  await expect(page.getByRole("dialog", { name: /^settings$/i })).toBeVisible();
}

test.describe("SettingsModal", () => {
  test("focus stays inside the dialog under Tab cycling", async ({ page }) => {
    await openSettings(page);
    const modal = page.getByRole("dialog", { name: /^settings$/i });

    const insideAtStart = await modal.evaluate((m) => m.contains(document.activeElement));
    expect(insideAtStart).toBe(true);

    for (let i = 0; i < 30; i++) {
      await page.keyboard.press("Tab");
      const inside = await modal.evaluate((m) => m.contains(document.activeElement));
      expect(inside).toBe(true);
    }
  });

  test("Escape closes the modal", async ({ page }) => {
    await openSettings(page);
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog", { name: /^settings$/i })).toHaveCount(0);
  });
});
