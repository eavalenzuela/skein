// Builds the addInitScript payload that installs the Tauri mock in the
// browser. The actual mock implementation lives in ./tauri-handlers.ts
// where it gets full TypeScript checking against the real
// Vault / Book / Page / Settings interfaces.
//
// We serialize installMock via Function.toString() and inject the
// JSON config as a literal arg. Because the function is serialized,
// it must not reference any outer-scope identifiers.

import { installMock, type MockConfig } from "./tauri-handlers";

export interface MockOptions {
  hasVault?: boolean;
  vaultName?: string;
  vaultRoot?: string;
}

export function tauriMockScript(opts: MockOptions = {}): string {
  const cfg: MockConfig = {
    hasVault: opts.hasVault ?? true,
    vaultName: opts.vaultName ?? "Test Vault",
    vaultRoot: opts.vaultRoot ?? "/tmp/test-vault",
  };
  return `(${installMock.toString()})(${JSON.stringify(cfg)});`;
}
