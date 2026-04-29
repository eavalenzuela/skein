<script lang="ts">
  import type { Page } from "../vault.js";
  import { createPage, renamePage, deletePage } from "../vault.js";
  import { openTab } from "../tabs.svelte.js";
  import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";
  import { focusTrap } from "../focusTrap.js";

  interface Props {
    pages: Page[];
    title: string;
    /** The book to create new pages into. `null` puts them at the vault
     * root (Folio). */
    book: string | null;
  }
  let { pages, title, book }: Props = $props();

  // Inline create
  let creating = $state(false);
  let createTitle = $state("");
  let createError = $state<string | null>(null);

  async function commitCreate() {
    const t = createTitle.trim();
    if (!t) {
      creating = false;
      createTitle = "";
      return;
    }
    try {
      const rel = await createPage(book, t);
      creating = false;
      createTitle = "";
      createError = null;
      // Open the new page right away.
      const stem = rel.split("/").pop()?.replace(/\.md$/, "") ?? rel;
      await openTab({ rel_path: rel, title: stem });
    } catch (e) {
      createError = String(e);
    }
  }

  // Inline rename
  let renamingRel = $state<string | null>(null);
  let renameValue = $state("");
  let renameError = $state<string | null>(null);

  async function commitRename() {
    const old = renamingRel;
    const next = renameValue.trim();
    if (!old || !next) {
      renamingRel = null;
      return;
    }
    const oldStem = old.split("/").pop()?.replace(/\.md$/, "") ?? "";
    if (next === oldStem) {
      renamingRel = null;
      return;
    }
    try {
      await renamePage(old, next);
      renamingRel = null;
      renameError = null;
    } catch (e) {
      renameError = String(e);
    }
  }

  // Delete confirmation
  let pendingDelete = $state<{ rel_path: string; title: string } | null>(null);
  let deleting = $state(false);
  let deleteError = $state<string | null>(null);

  async function confirmDelete() {
    if (!pendingDelete) return;
    deleting = true;
    deleteError = null;
    const target = pendingDelete;
    try {
      await deletePage(target.rel_path);
      pendingDelete = null;
    } catch (e) {
      deleteError = String(e);
    } finally {
      deleting = false;
    }
  }

  // Context menu
  let menu = $state<{ x: number; y: number; items: MenuItem[] } | null>(null);

  function openRowMenu(ev: MouseEvent, p: Page) {
    ev.preventDefault();
    ev.stopPropagation();
    menu = {
      x: ev.clientX,
      y: ev.clientY,
      items: [
        { label: "Open", action: () => openTab(p) },
        { separator: true, label: "", action: () => {} },
        {
          label: "Rename…",
          action: () => {
            renamingRel = p.rel_path;
            const stem = p.rel_path.split("/").pop()?.replace(/\.md$/, "") ?? p.title;
            renameValue = stem;
            renameError = null;
          },
        },
        {
          label: "Delete…",
          danger: true,
          action: () => {
            pendingDelete = { rel_path: p.rel_path, title: p.title };
          },
        },
      ],
    };
  }

  function openListMenu(ev: MouseEvent) {
    ev.preventDefault();
    menu = {
      x: ev.clientX,
      y: ev.clientY,
      items: [
        {
          label: "New page…",
          action: () => {
            creating = true;
            createTitle = "";
            createError = null;
          },
        },
      ],
    };
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="page-list" oncontextmenu={openListMenu}>
  <div class="hdr">
    <h2>{title}</h2>
    <div class="hdr-right">
      <span class="count">{pages.length} {pages.length === 1 ? "page" : "pages"}</span>
      <button
        class="add"
        onclick={() => {
          creating = true;
          createTitle = "";
          createError = null;
        }}
        title="New page"
        aria-label="New page">+</button
      >
    </div>
  </div>

  {#if creating}
    <div class="create-row">
      <!-- svelte-ignore a11y_autofocus -->
      <input
        bind:value={createTitle}
        autofocus
        placeholder="page title"
        onkeydown={(e) => {
          if (e.key === "Enter") void commitCreate();
          if (e.key === "Escape") {
            creating = false;
            createTitle = "";
            createError = null;
          }
        }}
        onblur={() => void commitCreate()}
      />
      {#if createError}
        <span class="err">{createError}</span>
      {/if}
    </div>
  {/if}

  {#if pages.length === 0 && !creating}
    <p class="empty">
      No pages yet. Click <strong>+</strong> to create one, or right-click anywhere here for the
      page menu.
    </p>
  {:else if pages.length > 0}
    <ul>
      {#each pages as p (p.rel_path)}
        <li>
          {#if renamingRel === p.rel_path}
            <div class="create-row">
              <!-- svelte-ignore a11y_autofocus -->
              <input
                bind:value={renameValue}
                autofocus
                onkeydown={(e) => {
                  if (e.key === "Enter") void commitRename();
                  if (e.key === "Escape") {
                    renamingRel = null;
                    renameError = null;
                  }
                }}
                onblur={() => void commitRename()}
              />
              {#if renameError}
                <span class="err">{renameError}</span>
              {/if}
            </div>
          {:else}
            <button
              onclick={() => openTab(p)}
              oncontextmenu={(e) => openRowMenu(e, p)}
              draggable="true"
              title="Click to open · drag to a pane to pin · right-click for rename / delete"
              ondragstart={(e) => {
                if (!e.dataTransfer) return;
                const payload = JSON.stringify({ rel_path: p.rel_path, title: p.title });
                e.dataTransfer.setData("application/x-skein-page", payload);
                e.dataTransfer.effectAllowed = "copyMove";
              }}
            >
              <span class="ttl">{p.title}</span>
              <span class="rp">{p.rel_path}</span>
            </button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if menu}
  <ContextMenu x={menu.x} y={menu.y} items={menu.items} onclose={() => (menu = null)} />
{/if}

{#if pendingDelete}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="overlay"
    onclick={() => {
      if (!deleting) pendingDelete = null;
    }}
    onkeydown={(e) => e.key === "Escape" && !deleting && (pendingDelete = null)}
  >
    <div
      class="modal"
      role="dialog"
      aria-label="Delete page"
      aria-modal="true"
      tabindex="-1"
      use:focusTrap
      onclick={(e) => e.stopPropagation()}
    >
      <h3>Delete "{pendingDelete.title}"?</h3>
      <p>This removes the file from disk and drops it from the index.</p>
      <div class="actions">
        <button onclick={() => (pendingDelete = null)}>Cancel</button>
        <button class="danger" onclick={confirmDelete} disabled={deleting}>
          Delete
        </button>
      </div>
      {#if deleteError}<p class="err">{deleteError}</p>{/if}
    </div>
  </div>
{/if}

<style>
  .page-list {
    flex: 1;
    overflow: auto;
    padding: 22px 38px;
    background: var(--paper);
    color: var(--ink);
    font-family: "Inter", system-ui, sans-serif;
  }
  .hdr {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    border-bottom: 1px solid var(--page-edge);
    padding-bottom: 8px;
    margin-bottom: 16px;
  }
  .hdr-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  h2 {
    font-family: "Source Serif 4", Georgia, serif;
    font-weight: 600;
    margin: 0;
    font-size: 20px;
    color: var(--ink);
  }
  .count {
    color: var(--ink-3);
    font-size: 11.5px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .add {
    background: transparent;
    color: var(--ink-3);
    border: 1px solid var(--page-edge);
    border-radius: 4px;
    width: 22px;
    height: 22px;
    line-height: 1;
    font-size: 14px;
    cursor: pointer;
    padding: 0;
  }
  .add:hover {
    color: var(--ink);
    border-color: var(--accent-edge, oklch(0.78 0.13 75));
  }
  .empty {
    color: var(--ink-3);
    font-size: 12px;
  }
  .create-row {
    margin: 6px 0 14px;
  }
  .create-row input {
    width: 100%;
    background: var(--page);
    color: var(--ink);
    border: 1px solid var(--accent-edge, oklch(0.78 0.13 75));
    border-radius: 4px;
    padding: 7px 10px;
    font: inherit;
    font-size: 13px;
  }
  .err {
    display: block;
    margin-top: 4px;
    font-size: 11px;
    color: oklch(0.7 0.16 25);
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  li {
    margin: 0;
  }
  button {
    background: transparent;
    color: inherit;
    border: 0;
    border-bottom: 1px solid var(--page-edge);
    padding: 12px 4px;
    width: 100%;
    text-align: left;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 3px;
    font: inherit;
  }
  button:hover {
    background: oklch(from var(--paper) calc(l + 0.02) c h);
  }
  .ttl {
    font-family: "Source Serif 4", Georgia, serif;
    font-size: 14.5px;
    color: var(--ink);
  }
  .rp {
    font-family: "JetBrains Mono", monospace;
    font-size: 11px;
    color: var(--ink-4);
  }

  .overlay {
    position: fixed;
    inset: 0;
    background: oklch(0 0 0 / 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
  }
  .modal {
    background: var(--chrome-2, #2a2a2c);
    color: var(--ink, #e6e6e6);
    border: 1px solid var(--chrome-edge, #3a3a3d);
    border-radius: 8px;
    padding: 22px 26px;
    max-width: 440px;
    box-shadow: 0 30px 80px -20px oklch(0 0 0 / 0.6);
  }
  .modal h3 {
    margin: 0 0 10px;
    font-family: "Source Serif 4", serif;
    font-weight: 600;
  }
  .modal p {
    margin: 0 0 18px;
    font-size: 13px;
    color: oklch(0.78 0.015 75);
    line-height: 1.5;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }
  .modal button {
    background: var(--chrome-3, #333335);
    color: var(--ink, #e6e6e6);
    border: 1px solid var(--chrome-edge, #3a3a3d);
    border-radius: 5px;
    padding: 7px 14px;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .modal button:hover {
    filter: brightness(1.1);
  }
  .modal button.danger {
    background: oklch(0.45 0.18 25);
    color: oklch(0.96 0.02 60);
    border-color: oklch(0.55 0.18 25);
  }
  .modal button:disabled {
    opacity: 0.6;
    cursor: progress;
  }
</style>
