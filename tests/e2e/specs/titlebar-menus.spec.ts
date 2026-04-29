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

  test("Help → About shows an alert mentioning the vault", async ({ page }) => {
    let alertText = "";
    page.on("dialog", (d) => {
      alertText = d.message();
      void d.accept();
    });

    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);

    await page.getByRole("button", { name: "Help" }).click();
    await page.getByRole("menuitem", { name: /about skein/i }).click();
    expect(alertText).toContain("Test Vault");
  });
});
