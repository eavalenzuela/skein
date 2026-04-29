<script lang="ts">
  import "./styles.css";
  import type { Theme, ShelfStyle, SidebarMode, PageFont } from "./tweaks.svelte.js";
  import { vaultState, selectBook } from "./vault.svelte.js";
  import {
    tabsState,
    setActive,
    closeTab,
    pinned,
    activeTab,
    openTab,
    replaceAtPin,
    selectInPane,
    bookOf,
    type Tab,
  } from "./tabs.svelte.js";
  import { searchUi, openSearch, closeSearch } from "./searchUi.svelte.js";
  import { settingsUi, openSettings, closeSettings } from "./settingsUi.svelte.js";
  import Titlebar from "./components/Titlebar.svelte";
  import VaultBookshelf from "./components/VaultBookshelf.svelte";
  import LiveTabs from "./components/LiveTabs.svelte";
  import EditorPage from "./components/EditorPage.svelte";
  import PinPlaceholder from "./components/PinPlaceholder.svelte";
  import PageList from "./components/PageList.svelte";
  import EmptyDesk from "./components/EmptyDesk.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import RelatedStrip from "./components/RelatedStrip.svelte";
  import LinkedFromStrip from "./components/LinkedFromStrip.svelte";
  import SettingsModal from "./components/SettingsModal.svelte";

  function onKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "K")) {
      e.preventDefault();
      if (searchUi.open) closeSearch();
      else openSearch();
    } else if ((e.ctrlKey || e.metaKey) && (e.key === "p" || e.key === "P") && !e.shiftKey) {
      e.preventDefault();
      openSearch();
    } else if ((e.ctrlKey || e.metaKey) && e.key === ",") {
      e.preventDefault();
      if (settingsUi.open) closeSettings();
      else openSettings();
    }
  }

  interface Props {
    theme?: Theme;
    shelfStyle?: ShelfStyle;
    sidebar?: SidebarMode;
    pageFont?: PageFont;
  }
  let {
    theme = "dark",
    shelfStyle = "suggestive",
    sidebar = "open",
    pageFont = "Source Serif 4",
  }: Props = $props();

  let midClass = $derived(
    sidebar === "open"
      ? "sidebar-open"
      : sidebar === "collapsed"
        ? "sidebar-collapsed"
        : "sidebar-hidden",
  );

  let leftPin = $derived(pinned("left"));
  let rightPin = $derived(pinned("right"));
  let isSplit = $derived(!!leftPin || !!rightPin);
  let active = $derived(activeTab());

  // Tab to render in the unpinned pane: the active tab when it's not the
  // tab pinned to the *other* side. Otherwise the unpinned pane is empty.
  function unpinnedPaneTab(otherSidePin: typeof leftPin) {
    if (!active) return undefined;
    if (otherSidePin && otherSidePin.rel_path === active.rel_path) return undefined;
    return active;
  }

  let leftPaneTab = $derived(leftPin ?? unpinnedPaneTab(rightPin));
  let rightPaneTab = $derived(rightPin ?? unpinnedPaneTab(leftPin));
  let leftBook = $derived(leftPaneTab ? bookOf(leftPaneTab.rel_path) : undefined);
  let rightBook = $derived(rightPaneTab ? bookOf(rightPaneTab.rel_path) : undefined);

  // Per-pane tab bar fires only when both panes hold tabs from *different*
  // books. Same-book or single-pane scenarios keep the shared strip.
  let splitTabBar = $derived(
    !!leftPaneTab && !!rightPaneTab && leftBook !== rightBook,
  );

  function tabsForBook(book: string | null | undefined): Tab[] {
    if (book === undefined) return [];
    return tabsState.tabs.filter((t) => bookOf(t.rel_path) === book);
  }

  function parsePagePayload(dt: DataTransfer | null) {
    if (!dt) return null;
    const raw = dt.getData("application/x-skein-page");
    if (!raw) return null;
    try {
      const parsed = JSON.parse(raw) as { rel_path: string; title: string };
      if (parsed.rel_path && parsed.title) return parsed;
    } catch {
      /* fallthrough */
    }
    return null;
  }

  let dragSide = $state<"left" | "right" | null>(null);

  function onPaneDragOver(ev: DragEvent, side: "left" | "right") {
    if (!ev.dataTransfer) return;
    const types = Array.from(ev.dataTransfer.types ?? []);
    const accepts =
      types.includes("application/x-skein-page") ||
      types.includes("application/x-skein-book");
    if (!accepts) return;
    ev.preventDefault();
    ev.dataTransfer.dropEffect = "copy";
    dragSide = side;
  }

  async function onPaneDrop(ev: DragEvent, side: "left" | "right") {
    const dt = ev.dataTransfer;
    if (!dt) return;
    let payload = parsePagePayload(dt);
    if (!payload) {
      const bookName = dt.getData("application/x-skein-book");
      if (bookName) {
        const { listPagesInBook } = await import("./vault.js");
        const pages = await listPagesInBook(bookName);
        const first = pages[0];
        if (first) {
          payload = { rel_path: first.rel_path, title: first.title };
        } else {
          // Book has no pages — nothing to open.
          ev.preventDefault();
          dragSide = null;
          return;
        }
      }
    }
    if (!payload) return;
    ev.preventDefault();
    dragSide = null;
    const sidePin = side === "left" ? leftPin : rightPin;
    if (sidePin) {
      await replaceAtPin(side, payload);
    } else {
      await openTab(payload);
    }
  }

  function onPaneDragLeave(side: "left" | "right") {
    if (dragSide === side) dragSide = null;
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- Modals must render inside .skein so the theme CSS variables are
     in scope; otherwise they resolve to nothing and text comes out
     black on the dark backdrop. -->
<div class="skein theme-{theme}" style:--page-font={pageFont}>
  <div class="win">
    <Titlebar vault={vaultState.vault?.name ?? "Skein"} />
    <div class="sk-body">
      <VaultBookshelf style={shelfStyle} {theme} books={vaultState.books} />
      <div class="sk-mid {midClass}" style:position="relative">
        <div class="sk-desk">
          {#if tabsState.tabs.length > 0}
            {#if !splitTabBar}
              <LiveTabs
                tabs={tabsState.tabs}
                activeId={tabsState.activeId}
                onSelect={setActive}
                onClose={closeTab}
              />
            {/if}
            <div class="sk-surface" class:split-bar={splitTabBar}>
              {#if isSplit}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="pane"
                  class:drag-over={dragSide === "left"}
                  ondragover={(e) => onPaneDragOver(e, "left")}
                  ondragleave={() => onPaneDragLeave("left")}
                  ondrop={(e) => onPaneDrop(e, "left")}
                >
                  {#if splitTabBar && leftPaneTab}
                    <LiveTabs
                      tabs={tabsForBook(leftBook)}
                      activeId={leftPaneTab.rel_path}
                      onSelect={(rp) => selectInPane("left", rp)}
                      onClose={closeTab}
                    />
                  {/if}
                  {#if leftPaneTab}
                    <EditorPage tab={leftPaneTab} />
                  {:else}
                    <PinPlaceholder side="left" />
                  {/if}
                </div>
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="pane"
                  class:drag-over={dragSide === "right"}
                  ondragover={(e) => onPaneDragOver(e, "right")}
                  ondragleave={() => onPaneDragLeave("right")}
                  ondrop={(e) => onPaneDrop(e, "right")}
                >
                  {#if splitTabBar && rightPaneTab}
                    <LiveTabs
                      tabs={tabsForBook(rightBook)}
                      activeId={rightPaneTab.rel_path}
                      onSelect={(rp) => selectInPane("right", rp)}
                      onClose={closeTab}
                    />
                  {/if}
                  {#if rightPaneTab}
                    <EditorPage tab={rightPaneTab} />
                  {:else}
                    <PinPlaceholder side="right" />
                  {/if}
                </div>
              {:else if active}
                <EditorPage tab={active} />
              {/if}
            </div>
            <LinkedFromStrip />
            <RelatedStrip />
          {:else if vaultState.activeBook}
            <div class="sk-tabs-empty">
              <button class="bare-link" onclick={() => selectBook(null)}>
                &laquo; back to Folio
              </button>
            </div>
            <PageList
              title={vaultState.activeBook}
              pages={vaultState.pagesInActiveBook}
              book={vaultState.activeBook}
            />
          {:else}
            <div class="sk-tabs-empty">No tabs open</div>
            {#if vaultState.loosePages.length > 0}
              <PageList
                title="Folio — loose pages"
                pages={vaultState.loosePages}
                book={null}
              />
            {:else}
              <EmptyDesk />
            {/if}
          {/if}
        </div>
        <Sidebar mode={sidebar} />
      </div>
    </div>
  </div>

  {#if searchUi.open}
    <CommandPalette />
  {/if}

  {#if settingsUi.open}
    <SettingsModal onClose={closeSettings} />
  {/if}
</div>

<style>
  .skein {
    width: 100%;
    height: 100%;
  }
  .bare-link {
    background: transparent;
    border: 0;
    cursor: pointer;
    color: var(--ink-3);
    font-family: "Inter", sans-serif;
    font-size: 12px;
    padding-left: 18px;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .bare-link:hover {
    color: var(--ink);
  }
  .pane {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    transition: outline-color 0.12s ease;
    outline: 2px solid transparent;
    outline-offset: -2px;
  }
  .pane.drag-over {
    outline-color: var(--accent-edge, oklch(0.78 0.13 75));
  }
  /* In split-tab mode each pane has its own LiveTabs strip on top, so the
     surface no longer needs the implicit gap below the shared strip. */
  .sk-surface.split-bar {
    /* hook for any future tweaks; layout falls out of .pane already */
  }
</style>
