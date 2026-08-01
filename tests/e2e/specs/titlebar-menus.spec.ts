import { test, expect } from "../fixtures";

test.describe("Titlebar menus", () => {
  test("File menu opens a context menu and supports keyboard nav", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);

    await page.getByRole("button", { name: "File" }).click();
    const menu = page.getByRole("menu");
    await expect(menu).toBeVisible();
    await expect(menu.getByRole("menuitem", { name: /new page/i })).toBeVisible();
    await expect(menu.getByRole("menuitem", { name: /switch vault/i })).toBeVisible();

    // Arrow Down then Enter should activate the second selectable item.
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Escape");
    await expect(menu).toHaveCount(0);
  });

  test("Help → About shows a toast mentioning the vault", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);

    await page.getByRole("button", { name: "Help" }).click();
    await page.getByRole("menuitem", { name: /about skein/i }).click();
    // In-app toast rather than a blocking window.alert (which is a no-op
    // in macOS WKWebView).
    await expect(page.locator(".toast")).toContainText("Test Vault");
  });

  test("Help → Keyboard shortcuts opens the in-app overlay", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);

    await page.getByRole("button", { name: "Help" }).click();
    await page.getByRole("menuitem", { name: /keyboard shortcuts/i }).click();
    const sheet = page.getByRole("dialog", { name: /keyboard shortcuts/i });
    await expect(sheet).toBeVisible();
    await expect(sheet).toContainText("Search pages and commands");
    await page.keyboard.press("Escape");
    await expect(sheet).toHaveCount(0);
  });

  test("Ctrl+/ toggles the shortcuts overlay", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    // Put focus in the document before sending a global shortcut.
    await page.locator(".sk-desk").click({ position: { x: 5, y: 5 } });

    await page.keyboard.press("Control+/");
    await expect(page.getByRole("dialog", { name: /keyboard shortcuts/i })).toBeVisible();
    await page.keyboard.press("Control+/");
    await expect(page.getByRole("dialog", { name: /keyboard shortcuts/i })).toHaveCount(0);
  });
});
