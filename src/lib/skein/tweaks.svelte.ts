// Reactive appearance + dev-preview state, persisted to disk via the
// Phase 6 settings commands. Scenario is dev-only and not persisted —
// it drives the design preview, not real vault behavior.

import { getSettings, setSettings, type Settings } from "./settings.js";

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

const VALID_THEMES: Theme[] = ["dark", "light"];
const VALID_SHELVES: ShelfStyle[] = ["abstract", "suggestive", "tactile"];
const VALID_SIDEBARS: SidebarMode[] = ["open", "collapsed", "hidden"];
const VALID_FONTS: PageFont[] = ["Source Serif 4", "Iowan Old Style", "Spectral", "Lora"];

let bootstrapped = false;

function applyLoaded(s: Settings) {
  if (s.theme && (VALID_THEMES as string[]).includes(s.theme)) tweaks.theme = s.theme as Theme;
  if (s.shelf_style && (VALID_SHELVES as string[]).includes(s.shelf_style))
    tweaks.shelfStyle = s.shelf_style as ShelfStyle;
  if (s.sidebar && (VALID_SIDEBARS as string[]).includes(s.sidebar))
    tweaks.sidebar = s.sidebar as SidebarMode;
  if (s.page_font && (VALID_FONTS as string[]).includes(s.page_font))
    tweaks.pageFont = s.page_font as PageFont;
}

export async function bootstrap() {
  try {
    applyLoaded(await getSettings());
  } catch {
    // ignore — defaults stay in place
  }
  bootstrapped = true;
}

let saveTimer: ReturnType<typeof setTimeout> | null = null;

export function persist() {
  if (!bootstrapped) return;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    void setSettings({
      theme: tweaks.theme,
      shelf_style: tweaks.shelfStyle,
      sidebar: tweaks.sidebar,
      page_font: tweaks.pageFont,
    }).catch(() => {});
  }, 200);
}
