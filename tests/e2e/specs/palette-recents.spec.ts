import { test, expect } from "../fixtures";

test.describe("Command palette zero-state", () => {
  test("teaches the : and # prefixes when there is nothing to show yet", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    await page.getByRole("button", { name: /^search$/i }).click();

    const dialog = page.getByRole("dialog", { name: /search pages/i });
    await expect(dialog.locator(".zero")).toContainText("run a command");
    await expect(dialog.locator(".zero")).toContainText("find pages by tag");
  });

  test("lists recently opened pages, most recent first", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    await page.getByRole("button", { name: /open research/i }).click();
    await page.locator(".page-list").getByText("Alpha note").click();
    await expect(page.locator(".cm-content")).toBeVisible();

    await page.getByRole("button", { name: /^search$/i }).click();
    const dialog = page.getByRole("dialog", { name: /search pages/i });
    await expect(dialog.locator(".hint-row")).toContainText("recently opened");
    await expect(dialog.locator(".results li").first()).toContainText("Alpha note");
    // No snippet row on a recent — there was no search to highlight.
    await expect(dialog.locator(".results li").first().locator(".snip")).toHaveCount(0);
  });

  test("a recent opens straight from the empty palette", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    await page.getByRole("button", { name: /open research/i }).click();
    await page.locator(".page-list").getByText("Beta note").click();
    await expect(page.locator(".cm-content")).toBeVisible();
    await page.locator(".sk-tab").filter({ hasText: "Beta note" }).locator(".x, [aria-label*=lose]").first().click();

    await page.getByRole("button", { name: /^search$/i }).click();
    await page.getByRole("dialog", { name: /search pages/i }).locator(".results li").first().click();
    await expect(page.locator(".sk-tab").filter({ hasText: "Beta note" })).toBeVisible();
  });
});
