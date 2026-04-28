// Typed wrappers around the Phase 6 settings + secrets commands.

import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  vault_path: string | null;
  theme: string | null;
  shelf_style: string | null;
  sidebar: string | null;
  page_font: string | null;
  daily_book: string | null;
  daily_template: string | null;
  daily_reminder_time: string | null;
}

export interface SettingsPatch {
  theme?: string;
  shelf_style?: string;
  sidebar?: string;
  page_font?: string;
  daily_book?: string;
  daily_template?: string;
  daily_reminder_time?: string;
}

export const getSettings = () => invoke<Settings>("get_settings");
export const setSettings = (patch: SettingsPatch) => invoke<Settings>("set_settings", { patch });

export const hasSecret = (name: string) => invoke<boolean>("has_secret", { name });
export const setSecret = (name: string, value: string) =>
  invoke<void>("set_secret", { name, value });
export const clearSecret = (name: string) => invoke<void>("clear_secret", { name });

export type SecretName = "anthropic_api_key" | "voyage_api_key";
