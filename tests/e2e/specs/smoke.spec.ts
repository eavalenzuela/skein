import { test, expect } from "../fixtures";

test("app boots and renders the bookshelf with mock vault", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(String(e)));

  await page.goto("/");
  // Bootstrap promise resolves and the loading "…" disappears.
  await expect(page.locator(".loading")).toHaveCount(0, { timeout: 10_000 });

  // Mock vault has Research / Daily / Recipes.
  await expect(page.getByText("Research")).toBeVisible();
  await expect(page.getByText("Daily")).toBeVisible();
  await expect(page.getByText("Recipes")).toBeVisible();

  expect(errors).toEqual([]);
});

test("vault picker shows when no vault is open", async ({ page }) => {
  await page.addInitScript(() => {
    // override: drop the pre-set vault so VaultPicker shows
    const w = window as any;
    const orig = w.__TAURI_INTERNALS__;
    if (orig) {
      const wrapped = { ...orig };
      const origInvoke = orig.invoke;
      wrapped.invoke = (cmd: string, args: any) => {
        if (cmd === "current_vault") return Promise.resolve(null);
        return origInvoke(cmd, args);
      };
      w.__TAURI_INTERNALS__ = wrapped;
    }
  });

  await page.goto("/");
  await expect(page.locator(".loading")).toHaveCount(0, { timeout: 10_000 });
  // VaultPicker should be visible. The "preview" link is also visible in this state.
  await expect(page.getByText(/skip — see the design preview/i)).toBeVisible();
});
