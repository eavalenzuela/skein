import { test, expect } from "../fixtures";

test.describe("Settings unsaved-changes guard", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    await page.getByRole("button", { name: /^settings/i }).first().click();
    await expect(page.getByRole("dialog", { name: /^settings$/i })).toBeVisible();
  });

  test("closes straight away when nothing is dirty", async ({ page }) => {
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog", { name: /^settings$/i })).toHaveCount(0);
  });

  test("warns before discarding an unsaved sync edit", async ({ page }) => {
    const settings = page.getByRole("dialog", { name: /^settings$/i });
    await settings.locator(".grid > input[type=text]").first().fill("https://github.com/me/notes.git");

    await page.keyboard.press("Escape");
    const warn = page.getByRole("alertdialog", { name: /unsaved settings/i });
    await expect(warn).toContainText("Sync");
    // Settings is still open behind it.
    await expect(settings).toBeVisible();

    await warn.getByRole("button", { name: /keep editing/i }).click();
    await expect(warn).toHaveCount(0);
    await expect(settings).toBeVisible();
    await expect(settings.locator(".grid > input[type=text]").first()).toHaveValue(
      "https://github.com/me/notes.git",
    );
  });

  test("discard closes the modal", async ({ page }) => {
    const settings = page.getByRole("dialog", { name: /^settings$/i });
    await settings.locator(".grid > input[type=text]").first().fill("https://example.com/x.git");
    await page.keyboard.press("Escape");
    await page.getByRole("alertdialog").getByRole("button", { name: /discard/i }).click();
    await expect(settings).toHaveCount(0);
  });

  test("saving clears the dirty state", async ({ page }) => {
    const settings = page.getByRole("dialog", { name: /^settings$/i });
    await settings.locator(".grid > input[type=text]").first().fill("https://github.com/me/n.git");
    await settings.getByRole("button", { name: /^save$/i }).first().click();
    await expect(settings.getByText(/^Saved\.$/)).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(settings).toHaveCount(0);
  });
});

test("Escape still closes Settings after clicking a button that disables itself", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator(".loading")).toHaveCount(0);
  await page.getByRole("button", { name: /^settings/i }).first().click();
  const settings = page.getByRole("dialog", { name: /^settings$/i });

  // "save" disables itself while busy, which moves focus to <body>; a
  // handler bound to the overlay would never see the keypress.
  await settings.getByRole("button", { name: /refresh status/i }).click();
  await page.keyboard.press("Escape");
  await expect(settings).toHaveCount(0);
});

test("Ctrl+, goes through the unsaved-changes guard too", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".loading")).toHaveCount(0);
  await page.getByRole("button", { name: /^settings/i }).first().click();
  const settings = page.getByRole("dialog", { name: /^settings$/i });
  await settings.locator(".grid > input[type=text]").first().fill("https://github.com/me/n.git");

  // Ctrl+, is the advertised way in, so it must not be the one way out
  // that discards edits silently.
  await page.keyboard.press("Control+,");
  await expect(page.getByRole("alertdialog", { name: /unsaved settings/i })).toBeVisible();
  await expect(settings).toBeVisible();
});

test("one Escape dismisses only the frontmost dialog", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".loading")).toHaveCount(0);
  await page.getByRole("button", { name: /^settings/i }).first().click();
  const settings = page.getByRole("dialog", { name: /^settings$/i });
  await expect(settings).toBeVisible();

  await page.keyboard.press("Control+/");
  const shortcuts = page.getByRole("dialog", { name: /keyboard shortcuts/i });
  await expect(shortcuts).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(shortcuts).toHaveCount(0);
  await expect(settings).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(settings).toHaveCount(0);
});
