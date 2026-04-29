import { test, expect } from "../fixtures";

test.describe("Page CRUD via PageList", () => {
  test("opening a book shows its pages, and + creates a new one", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);

    // Click into Research to show its PageList.
    await page.getByRole("button", { name: /open research/i }).click();
    await expect(page.locator(".page-list")).toBeVisible();
    await expect(page.locator(".page-list").getByText("Alpha note")).toBeVisible();
    await expect(page.locator(".page-list").getByText("Beta note")).toBeVisible();

    // Create a new page.
    await page.locator(".page-list .add").click();
    const input = page.locator(".create-row input").first();
    await input.fill("Gamma");
    await input.press("Enter");
    // The new page opens in a tab; the PageList is replaced by the editor.
    await expect(page.locator(".sk-tab").filter({ hasText: "Gamma" })).toBeVisible();
  });

  test("right-click on a page row exposes Rename / Delete", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    await page.getByRole("button", { name: /open research/i }).click();

    const row = page
      .locator(".page-list li button")
      .filter({ hasText: "Alpha note" })
      .first();
    await row.click({ button: "right" });
    const menu = page.getByRole("menu");
    await expect(menu.getByRole("menuitem", { name: /rename/i })).toBeVisible();
    await expect(menu.getByRole("menuitem", { name: /delete/i })).toBeVisible();
  });
});
