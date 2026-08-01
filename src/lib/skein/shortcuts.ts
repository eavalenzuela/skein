// Single source of truth for keyboard shortcuts and interaction hints.
//
// These used to live in three places that drifted apart: the handler in
// VaultWindow, the palette's hint row, and a window.alert list in the Help
// menu. The overlay and the Help menu now both render from here, so adding
// a binding in one place documents it everywhere.

export interface Shortcut {
  keys: string[];
  label: string;
}

export interface ShortcutGroup {
  title: string;
  items: Shortcut[];
}

export const SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    title: "Anywhere",
    items: [
      { keys: ["Ctrl", "K"], label: "Search pages and commands" },
      { keys: ["Ctrl", "P"], label: "Search pages (same palette)" },
      { keys: ["Ctrl", "D"], label: "Open today's daily note" },
      { keys: ["Ctrl", ","], label: "Settings" },
      { keys: ["Ctrl", "/"], label: "This shortcut list" },
      { keys: ["Esc"], label: "Close the open dialog or menu" },
    ],
  },
  {
    title: "Command palette",
    items: [
      { keys: ["↑", "↓"], label: "Move through results" },
      { keys: ["Enter"], label: "Open the highlighted result" },
      { keys: [":"], label: "Run a command instead of searching" },
      { keys: ["#"], label: "Find pages by tag" },
    ],
  },
  {
    title: "Editor",
    items: [
      { keys: ["[", "["], label: "Start a wikilink, with autocomplete" },
      { keys: ["Ctrl", "V"], label: "Paste an image as an attachment" },
    ],
  },
  {
    title: "Chat",
    items: [
      { keys: ["Enter"], label: "Send" },
      { keys: ["Shift", "Enter"], label: "New line" },
      { keys: ["@"], label: "Mention a page to pull it into context" },
    ],
  },
  {
    title: "Mouse",
    items: [
      { keys: ["Right-click"], label: "Context menu on a book or page" },
      { keys: ["Drag"], label: "Reorder shelf spines, or drop a page into a pane" },
      { keys: ["Drag"], label: "Pull a chat selection into the editor" },
    ],
  },
];
