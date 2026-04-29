<script lang="ts">
  import { openSearch } from "../searchUi.svelte.js";
  import { openSettings } from "../settingsUi.svelte.js";
  import { openTodayDaily, createPage, rebuildIndex } from "../vault.js";
  import { openTab, closeTab, tabsState } from "../tabs.svelte.js";
  import { close as closeVault, vaultState } from "../vault.svelte.js";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";

  interface Props {
    vault: string;
  }
  let { vault }: Props = $props();

  type MenuName = "file" | "edit" | "view" | "help";
  let openMenu = $state<MenuName | null>(null);
  let menuPos = $state<{ x: number; y: number }>({ x: 0, y: 0 });

  function showMenu(name: MenuName, e: MouseEvent) {
    const t = e.currentTarget as HTMLElement;
    const rect = t.getBoundingClientRect();
    menuPos = { x: rect.left, y: rect.bottom + 2 };
    openMenu = name;
  }
  function closeMenu() {
    openMenu = null;
  }

  async function newPagePrompt() {
    const title = window.prompt("Title for the new page:");
    if (!title?.trim()) return;
    const rel = await createPage(null, title.trim());
    await openTab({ rel_path: rel, title: title.trim() });
  }

  function activeTabRel(): string | null {
    return tabsState.activeId;
  }

  const fileItems: MenuItem[] = [
    { label: "New page…", action: () => void newPagePrompt() },
    { label: "Today's daily note", action: () => void jumpToToday() },
    { separator: true, label: "", action: () => {} },
    { label: "Switch vault…", action: () => void closeVault() },
    { separator: true, label: "", action: () => {} },
    { label: "Quit", action: () => void winClose() },
  ];
  const editItems: MenuItem[] = [
    {
      label: "Undo",
      action: () => document.execCommand?.("undo"),
    },
    {
      label: "Redo",
      action: () => document.execCommand?.("redo"),
    },
    { separator: true, label: "", action: () => {} },
    {
      label: "Cut",
      action: () => document.execCommand?.("cut"),
    },
    {
      label: "Copy",
      action: () => document.execCommand?.("copy"),
    },
    {
      label: "Paste",
      action: () => document.execCommand?.("paste"),
    },
    { separator: true, label: "", action: () => {} },
    { label: "Find in vault…", action: () => void openSearch() },
  ];
  const viewItems: MenuItem[] = [
    {
      label: "Close active tab",
      action: () => {
        const a = activeTabRel();
        if (a) closeTab(a);
      },
    },
    {
      label: "Rebuild search index",
      action: () => void rebuildIndex(),
    },
    { separator: true, label: "", action: () => {} },
    { label: "Settings…", action: () => openSettings() },
    {
      label: "Toggle full-screen",
      action: () => void toggleFullscreen(),
    },
  ];
  const helpItems: MenuItem[] = [
    {
      label: "Skein on GitHub",
      action: () => {
        window.open("https://github.com/anthropics/skein", "_blank");
      },
    },
    {
      label: "Keyboard shortcuts",
      action: () => {
        window.alert(
          "Skein keyboard shortcuts:\n\n" +
            "Ctrl+K — Search / command palette\n" +
            "Ctrl+, — Settings\n" +
            "Esc — Close modal\n" +
            "↑↓ Enter — Navigate / open in palette\n" +
            "Right-click — Context menu on books and pages\n" +
            "Drag book spine — Reorder shelf or open in pane",
        );
      },
    },
    { separator: true, label: "", action: () => {} },
    {
      label: "About Skein",
      action: () => {
        window.alert(
          "Skein — local note-taking with semantic search and an embedded Claude chat.\n\n" +
            `Vault: ${vaultState.vault?.name ?? "(none)"}\n` +
            `Path: ${vaultState.vault?.root ?? "(none)"}`,
        );
      },
    },
  ];

  function itemsFor(name: MenuName): MenuItem[] {
    switch (name) {
      case "file":
        return fileItems;
      case "edit":
        return editItems;
      case "view":
        return viewItems;
      case "help":
        return helpItems;
    }
  }

  async function toggleFullscreen() {
    const w = getCurrentWindow();
    const isFs = await w.isFullscreen();
    await w.setFullscreen(!isFs);
  }

  async function jumpToToday() {
    try {
      const res = await openTodayDaily();
      // Use the filename stem as the tab title — the indexer will refine it
      // once the watcher reconciles the new file.
      const stem = res.rel_path.split("/").pop()?.replace(/\.md$/, "") ?? res.rel_path;
      await openTab({ rel_path: res.rel_path, title: stem });
    } catch {
      // No vault open or write failed — silently ignore for now; the
      // Settings modal is the right place to surface daily-note errors.
    }
  }

  async function winMinimize() {
    await getCurrentWindow().minimize();
  }
  async function winToggleMaximize() {
    await getCurrentWindow().toggleMaximize();
  }
  async function winClose() {
    await getCurrentWindow().close();
  }
</script>

<div class="sk-titlebar">
  <div class="sk-tb-menus">
    {#each ["file", "edit", "view", "help"] as name (name)}
      <button
        class="sk-tb-menu bare"
        class:active={openMenu === name}
        onclick={(e) => showMenu(name as MenuName, e)}
      >
        {name[0].toUpperCase() + name.slice(1)}
      </button>
    {/each}
  </div>
  <div class="sk-tb-title">
    Skein <span class="vault">— {vault}</span>
  </div>
  <div class="sk-tb-right">
    <button
      class="sk-tb-btn bare"
      title="Today's daily note"
      onclick={jumpToToday}
      aria-label="Today's daily note"
    >
      <svg
        width="13"
        height="13"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.4"
      >
        <rect x="2.5" y="3.5" width="11" height="10" rx="1.5" />
        <path d="M5 2v3M11 2v3M2.5 7h11" />
      </svg>
    </button>
    <button class="sk-tb-btn bare" title="Search (Ctrl+K)" onclick={openSearch} aria-label="Search">
      <svg
        width="13"
        height="13"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.4"
      >
        <circle cx="7" cy="7" r="4" />
        <path d="M10 10l3 3" />
      </svg>
    </button>
    <button
      class="sk-tb-btn bare"
      title="Switch vault — close this one and return to the picker"
      onclick={() => void closeVault()}
      aria-label="Switch vault"
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.3"
      >
        <path d="M3 2.5h10M3 5.5h10M3 8.5h7M3 11.5h5" />
      </svg>
    </button>
    <button
      class="sk-tb-btn bare"
      title="Settings (Ctrl+,)"
      onclick={openSettings}
      aria-label="Settings"
    >
      <svg
        width="13"
        height="13"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.4"
      >
        <circle cx="8" cy="8" r="2.2" />
        <path
          d="M8 1.5v2M8 12.5v2M14.5 8h-2M3.5 8h-2M12.6 3.4l-1.4 1.4M4.8 11.2l-1.4 1.4M12.6 12.6l-1.4-1.4M4.8 4.8L3.4 3.4"
        />
      </svg>
    </button>
    <div class="sk-tb-window">
      <button title="Minimize" aria-label="Minimize" onclick={winMinimize}>
        <svg
          width="8"
          height="8"
          viewBox="0 0 8 8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
        >
          <path d="M1.5 4h5" />
        </svg>
      </button>
      <button title="Maximize" aria-label="Maximize" onclick={winToggleMaximize}>
        <svg
          width="8"
          height="8"
          viewBox="0 0 8 8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
        >
          <rect x="1.5" y="1.5" width="5" height="5" />
        </svg>
      </button>
      <button title="Close" class="close" aria-label="Close" onclick={winClose}>
        <svg
          width="8"
          height="8"
          viewBox="0 0 8 8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
        >
          <path d="M2 2l4 4M6 2l-4 4" />
        </svg>
      </button>
    </div>
  </div>
</div>

{#if openMenu}
  <ContextMenu x={menuPos.x} y={menuPos.y} items={itemsFor(openMenu)} onclose={closeMenu} />
{/if}

<style>
  .bare {
    background: transparent;
    border: 0;
    cursor: pointer;
    color: inherit;
    font: inherit;
  }
  .sk-tb-menu.bare {
    cursor: pointer;
  }
  .sk-tb-menu.active {
    background: oklch(1 0 0 / 0.08);
  }
</style>
