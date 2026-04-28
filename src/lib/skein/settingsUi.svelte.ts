// Lightweight global toggle for the Settings modal.

export const settingsUi: { open: boolean } = $state({ open: false });

export function openSettings() {
  settingsUi.open = true;
}
export function closeSettings() {
  settingsUi.open = false;
}
