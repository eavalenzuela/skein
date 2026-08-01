// Open/closed state for the keyboard-shortcuts overlay. Separate module so
// the Help menu and the global key handler can both drive it without
// importing each other.

export const shortcutsUi: { open: boolean } = $state({ open: false });

export const openShortcuts = () => (shortcutsUi.open = true);
export const closeShortcuts = () => (shortcutsUi.open = false);
export const toggleShortcuts = () => (shortcutsUi.open = !shortcutsUi.open);
