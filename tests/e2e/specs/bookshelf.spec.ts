import { test, expect } from "../fixtures";

test.describe("Bookshelf CRUD", () => {
  test("create a new book via the + slot", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);

    // The "+" slot lives in the bookshelf row after the existing books.
    await page.getByRole("button", { name: /new book/i }).click();
    const input = page.locator(".rename-slot input").first();
    await input.fill("Travel");
    await input.press("Enter");

    await expect(page.getByText("Travel")).toBeVisible();
  });

  test("rename a book via right-click", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);

    const research = page.getByRole("button", { name: /open research/i });
    await research.click({ button: "right" });
    await page.getByRole("menuitem", { name: /rename/i }).click();

    const input = page.locator(".rename-slot input").first();
    await input.fill("Studies");
    await input.press("Enter");

    await expect(page.getByText("Studies")).toBeVisible();
    await expect(page.getByText("Research")).toHaveCount(0);
  });

  test("type-to-jump focuses a book starting with the typed letter", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    const shelf = page.locator(".sk-shelf");
    await shelf.focus();
    await page.keyboard.press("R");
    // Research button has aria-label "Open Research"
    const focused = await page.evaluate(() => document.activeElement?.getAttribute("aria-label"));
    expect(focused).toMatch(/open research/i);
  });
});
