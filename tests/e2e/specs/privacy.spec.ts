import { test, expect } from "../fixtures";

test.describe("Auto-tagging opt-in", () => {
  test("never calls the API while the setting is off", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    await page.evaluate(() => {
      const w = window as unknown as {
        __SKEIN_MOCK__: {
          handlers: Record<string, (a: Record<string, unknown>) => unknown>;
          state: { secrets: Set<string> };
        };
        __suggestCalls: number;
      };
      // A key configured for chat must not by itself start uploading notes.
      w.__SKEIN_MOCK__.state.secrets.add("anthropic_api_key");
      w.__suggestCalls = 0;
      w.__SKEIN_MOCK__.handlers.suggest_tags = () => {
        w.__suggestCalls++;
        return ["embeddings"];
      };
    });

    await page.getByRole("button", { name: /open research/i }).click();
    await page.locator(".page-list").getByText("Alpha note").click();
    await expect(page.locator(".cm-content")).toBeVisible();
    await page.locator(".cm-content").click();
    await page.keyboard.type("a".repeat(60));
    // Well past the 3s auto-tag debounce.
    await page.waitForTimeout(4000);

    expect(
      await page.evaluate(() => (window as unknown as { __suggestCalls: number }).__suggestCalls),
    ).toBe(0);
    await expect(page.locator(".tag-chips")).toHaveCount(0);
  });

  test("Settings explains what leaves the machine and can turn it on", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator(".loading")).toHaveCount(0);
    await page.getByRole("button", { name: /^settings/i }).first().click();

    const settings = page.getByRole("dialog", { name: /^settings$/i });
    await expect(settings).toContainText(/live only on this\s+machine/i);
    await expect(settings).toContainText(/no telemetry/i);
    const toggle = settings.locator('input[type="checkbox"]').first();
    await expect(toggle).not.toBeChecked();
    await toggle.check();
    await expect(toggle).toBeChecked();

    // Persisted through the settings command, not just local component state.
    const saved = await page.evaluate(
      () =>
        (
          window as unknown as {
            __SKEIN_MOCK__: { state: { settings: { auto_tag?: boolean } } };
          }
        ).__SKEIN_MOCK__.state.settings.auto_tag,
    );
    expect(saved).toBe(true);
  });
});
