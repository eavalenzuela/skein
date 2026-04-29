import { test, expect } from "../fixtures";

// Visual regression snapshots. Re-run with `npx playwright test --update-snapshots`
// after intentional UI changes. Snapshots are platform-specific; commit only
// the linux ones produced by CI / local dev.

test.describe("Visual regression", () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
  });

  test("bookshelf with mock vault", async ({ page }) => {
    await expect(page).toHaveScreenshot("bookshelf.png", { fullPage: false });
  });

  test("page list opened from a book", async ({ page }) => {
    await page.getByRole("button", { name: /open research/i }).click();
    await expect(page.locator(".page-list")).toBeVisible();
    await expect(page).toHaveScreenshot("pagelist-research.png", { fullPage: false });
  });

  test("settings modal", async ({ page }) => {
    await page.getByRole("button", { name: /^settings/i }).first().click();
    await expect(page.getByRole("dialog", { name: /^settings$/i })).toBeVisible();
    await expect(page.getByRole("dialog", { name: /^settings$/i })).toHaveScreenshot(
      "settings-modal.png",
    );
  });

  test("command palette empty state", async ({ page }) => {
    await page.getByRole("button", { name: /^search$/i }).click();
    await expect(page.getByRole("dialog", { name: /search pages/i })).toBeVisible();
    await expect(page.getByRole("dialog", { name: /search pages/i })).toHaveScreenshot(
      "palette-empty.png",
    );
  });

  test("command palette in command mode", async ({ page }) => {
    await page.getByRole("button", { name: /^search$/i }).click();
    await page.getByRole("textbox", { name: /search pages or commands/i }).fill(":");
    await expect(page.locator(".results li").first()).toBeVisible();
    await expect(page.getByRole("dialog", { name: /search pages/i })).toHaveScreenshot(
      "palette-commands.png",
    );
  });

  test("titlebar File menu open", async ({ page }) => {
    await page.getByRole("button", { name: "File" }).click();
    await expect(page.getByRole("menu")).toBeVisible();
    await expect(page).toHaveScreenshot("titlebar-file-menu.png", { fullPage: false });
  });
});
