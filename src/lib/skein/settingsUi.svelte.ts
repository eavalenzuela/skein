// Lightweight global toggle for the Settings modal, plus the few persisted
// settings that components outside the modal need to react to.

import { getSettings, setSettings } from "./settings.js";

export const settingsUi: { open: boolean } = $state({ open: false });

export function openSettings() {
  settingsUi.open = true;
}
export function closeSettings() {
  settingsUi.open = false;
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
