// Phase 1 dev-only tweak store, mirroring the design's Tweaks panel.
// Phase 6 replaces this with persisted user settings.

export type Theme = "dark" | "light";
export type ShelfStyle = "abstract" | "suggestive" | "tactile";
export type SidebarMode = "open" | "collapsed" | "hidden";
export type Scenario = "populated" | "dragging" | "empty";
export type PageFont = "Source Serif 4" | "Iowan Old Style" | "Spectral" | "Lora";

export interface Tweaks {
  theme: Theme;
  shelfStyle: ShelfStyle;
  sidebar: SidebarMode;
  scenario: Scenario;
  pageFont: PageFont;
}

export const tweaks: Tweaks = $state({
  theme: "dark",
  shelfStyle: "suggestive",
  sidebar: "open",
  scenario: "populated",
  pageFont: "Source Serif 4",
});
