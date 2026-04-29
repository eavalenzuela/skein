import { test as base, expect } from "@playwright/test";
import { tauriMockScript, type MockOptions } from "./mocks/tauri-mock";

type Fixtures = {
  mockOptions: MockOptions;
  installMock: void;
};

export const test = base.extend<Fixtures>({
  mockOptions: [{ hasVault: true }, { option: true }],
  installMock: [
    async ({ page, mockOptions }, use) => {
      await page.addInitScript(tauriMockScript(mockOptions));
      await use();
    },
    { auto: true },
  ],
});

export { expect };
