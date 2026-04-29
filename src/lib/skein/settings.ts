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
  git_remote_url: string | null;
  git_branch: string | null;
  git_auth_kind: string | null;
}

export interface SettingsPatch {
  theme?: string;
  shelf_style?: string;
  sidebar?: string;
  page_font?: string;
  daily_book?: string;
  daily_template?: string;
  daily_reminder_time?: string;
  git_remote_url?: string;
  git_branch?: string;
  git_auth_kind?: string;
}

export const getSettings = () => invoke<Settings>("get_settings");
export const setSettings = (patch: SettingsPatch) => invoke<Settings>("set_settings", { patch });

export const hasSecret = (name: string) => invoke<boolean>("has_secret", { name });
export const setSecret = (name: string, value: string) =>
  invoke<void>("set_secret", { name, value });
export const clearSecret = (name: string) => invoke<void>("clear_secret", { name });

export type SecretName = "anthropic_api_key" | "voyage_api_key" | "git_token";

export interface GitDirtyFile {
  path: string;
  state: string;
}

export interface GitStatus {
  initialized: boolean;
  branch: string | null;
  remote_url: string | null;
  ahead: number;
  behind: number;
  dirty: GitDirtyFile[];
  conflicted: string[];
}

export interface GitPullResult {
  kind: "up-to-date" | "fast-forward" | "merged" | "conflicts";
  conflicted: string[];
}

export const gitStatus = () => invoke<GitStatus>("git_status");
export const gitSetRemote = (remoteUrl: string) =>
  invoke<void>("git_set_remote", { remoteUrl });
export const gitPull = () => invoke<GitPullResult>("git_pull");
export const gitPush = () => invoke<void>("git_push");
export const gitCommitAll = (message: string) =>
  invoke<boolean>("git_commit_all", { message });
