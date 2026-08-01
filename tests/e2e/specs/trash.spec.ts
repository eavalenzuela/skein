import { test, expect } from "../fixtures";

test.describe("Trash and undo", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    await page.getByRole("button", { name: /open research/i }).click();
    await expect(page.locator(".page-list")).toBeVisible();
  });

  async function deleteBeta(page: import("@playwright/test").Page) {
    await page
      .locator(".page-list li button")
      .filter({ hasText: "Beta note" })
      .first()
      .click({ button: "right" });
    await page.getByRole("menuitem", { name: /delete/i }).click();
    const dialog = page.getByRole("dialog", { name: /delete page/i });
    await expect(dialog).toContainText(/move .* to trash/i);
    await dialog.getByRole("button", { name: /move to trash/i }).click();
  }

  test("deleting a page offers an undo that brings it back", async ({ page }) => {
    await deleteBeta(page);
    await expect(page.locator(".page-list").getByText("Beta note")).toHaveCount(0);

    const toast = page.locator(".toast");
    await expect(toast).toContainText(/moved "beta note" to trash/i);
    await toast.getByRole("button", { name: /undo/i }).click();

    await expect(page.locator(".page-list").getByText("Beta note")).toBeVisible();
  });

  test("a deleted page stays recoverable from Settings", async ({ page }) => {
    await deleteBeta(page);
    // Dismiss the undo toast: Settings is the durable recovery path.
    await page.locator(".toast .x").first().click();

    await page.getByRole("button", { name: /^settings/i }).first().click();
    const settings = page.getByRole("dialog", { name: /^settings$/i });
    await expect(settings).toBeVisible();
    const row = settings.locator(".trash-list li").filter({ hasText: "Beta note" });
    await expect(row).toBeVisible();
    await row.getByRole("button", { name: /restore/i }).click();
    await expect(settings.locator(".trash-list li")).toHaveCount(0);

    await page.keyboard.press("Escape");
    await expect(page.locator(".page-list").getByText("Beta note")).toBeVisible();
  });

  test("emptying the trash clears the recoverable list", async ({ page }) => {
    await deleteBeta(page);
    await page.locator(".toast .x").first().click();

    await page.getByRole("button", { name: /^settings/i }).first().click();
    const settings = page.getByRole("dialog", { name: /^settings$/i });
    await expect(settings.locator(".trash-list li")).toHaveCount(1);
    await settings.getByRole("button", { name: /empty trash/i }).click();
    await expect(settings.getByText(/nothing in the trash/i)).toBeVisible();
  });
});
