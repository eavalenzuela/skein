// Lightweight global toggle for the Settings modal, plus the few persisted
// settings that components outside the modal need to react to.

import { getSettings, setSettings } from "./settings.js";

export const settingsUi: { open: boolean } = $state({ open: false });

/** Set by SettingsModal while it is mounted, so every close path — the X,
 * the backdrop, Escape, and Ctrl+, — goes through the same unsaved-changes
 * check. Without this the shortcut discarded edits silently. */
let closeGuard: (() => void) | null = null;

export function registerSettingsCloseGuard(fn: (() => void) | null) {
  closeGuard = fn;
}

export function openSettings() {
  settingsUi.open = true;
}

/** Close immediately, discarding anything unsaved. */
export function closeSettings() {
  settingsUi.open = false;
}

/** Ask to close: prompts first when a section has unsaved edits. */
export function requestCloseSettings() {
  if (closeGuard) closeGuard();
  else settingsUi.open = false;
}

/** Settings that gate behaviour elsewhere in the app. Kept here (rather
 * than read per-component) so a change in the modal takes effect
 * immediately without a reload. */
export const settingsState: { autoTag: boolean } = $state({ autoTag: false });

export async function bootstrapSettings() {
  try {
    const s = await getSettings();
    settingsState.autoTag = s.auto_tag === true;
  } catch {
    // Settings unreadable — leave the privacy-sensitive default (off).
  }
}

export async function setAutoTag(on: boolean) {
  settingsState.autoTag = on;
  await setSettings({ auto_tag: on });
}
