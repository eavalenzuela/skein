<script lang="ts">
  import "./styles.css";
  import type { Theme, ShelfStyle, SidebarMode, PageFont } from "./tweaks.svelte.js";
  import { vaultState, selectBook } from "./vault.svelte.js";
  import { tabsState, setActive, closeTab, togglePin, pinned, activeTab } from "./tabs.svelte.js";
  import Titlebar from "./components/Titlebar.svelte";
  import VaultBookshelf from "./components/VaultBookshelf.svelte";
  import LiveTabs from "./components/LiveTabs.svelte";
  import EditorPage from "./components/EditorPage.svelte";
  import PageList from "./components/PageList.svelte";
  import EmptyDesk from "./components/EmptyDesk.svelte";
  import Sidebar from "./components/Sidebar.svelte";

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
  let isSplit = $derived(!!leftPin && !!rightPin);
  let active = $derived(activeTab());
</script>

<div class="skein theme-{theme}" style:--page-font={pageFont}>
  <div class="win">
    <Titlebar vault={vaultState.vault?.name ?? "Skein"} />
    <div class="sk-body">
      <VaultBookshelf style={shelfStyle} {theme} books={vaultState.books} />
      <div class="sk-mid {midClass}" style:position="relative">
        <div class="sk-desk">
          {#if tabsState.tabs.length > 0}
            <LiveTabs
              tabs={tabsState.tabs}
              activeId={tabsState.activeId}
              onSelect={setActive}
              onClose={closeTab}
              onPin={togglePin}
            />
            <div class="sk-surface">
              {#if isSplit && leftPin && rightPin}
                <EditorPage tab={leftPin} />
                <EditorPage tab={rightPin} />
              {:else if active}
                <EditorPage tab={active} />
              {/if}
            </div>
          {:else if vaultState.activeBook}
            <div class="sk-tabs-empty">
              <button class="bare-link" onclick={() => selectBook(null)}>
                &laquo; back to Folio
              </button>
            </div>
            <PageList title={vaultState.activeBook} pages={vaultState.pagesInActiveBook} />
          {:else}
            <div class="sk-tabs-empty">No tabs open</div>
            {#if vaultState.loosePages.length > 0}
              <PageList title="Folio — loose pages" pages={vaultState.loosePages} />
            {:else}
              <EmptyDesk />
            {/if}
          {/if}
        </div>
        <Sidebar mode={sidebar} />
      </div>
    </div>
  </div>
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
</style>
