<script lang="ts">
  import "./styles.css";
  import type { Theme, ShelfStyle, SidebarMode, PageFont } from "./tweaks.svelte.js";
  import { vaultState, selectBook, closeOpenPage } from "./vault.svelte.js";
  import Titlebar from "./components/Titlebar.svelte";
  import VaultBookshelf from "./components/VaultBookshelf.svelte";
  import MarkdownPage from "./components/MarkdownPage.svelte";
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
</script>

<div class="skein theme-{theme}" style:--page-font={pageFont}>
  <div class="win">
    <Titlebar vault={vaultState.vault?.name ?? "Skein"} />
    <div class="sk-body">
      <VaultBookshelf style={shelfStyle} {theme} books={vaultState.books} />
      <div class="sk-mid {midClass}" style:position="relative">
        <div class="sk-desk">
          {#if vaultState.openPage}
            <div class="sk-tabs">
              <div class="sk-tab active">
                <span class="pin-ind">
                  <svg
                    width="10"
                    height="10"
                    viewBox="0 0 12 12"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.3"
                  >
                    <path d="M2.5 1.5h5l2 2v7h-7zM7 1.5v2h2" />
                  </svg>
                </span>
                <span class="name">{vaultState.openPage.title}</span>
                <button class="x bare" onclick={closeOpenPage} aria-label="Close page">×</button>
              </div>
            </div>
            <div class="sk-surface">
              <div class="sk-page">
                <MarkdownPage page={vaultState.openPage} />
              </div>
            </div>
          {:else if vaultState.activeBook}
            <div class="sk-tabs-empty">
              <button class="bare-link" onclick={() => selectBook(null)}
                >&laquo; back to Folio</button
              >
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
  .bare {
    background: transparent;
    border: 0;
    padding: 0;
    cursor: pointer;
    color: inherit;
    font: inherit;
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
