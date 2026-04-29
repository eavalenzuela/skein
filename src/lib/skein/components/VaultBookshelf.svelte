<script lang="ts">
  import type { Book } from "../vault.js";
  import { createBook, deleteBook, renameBook, setBookOrder } from "../vault.js";
  import type { ShelfStyle, Theme } from "../tweaks.svelte.js";
  import { spineHeight, spineHue, spineShade, spineWidth } from "../spineHash.js";
  import { vaultState, selectBook } from "../vault.svelte.js";
  import Spine from "./Spine.svelte";
  import Folio from "./Folio.svelte";
  import ContextMenu, { type MenuItem } from "./ContextMenu.svelte";

  interface Props {
    style: ShelfStyle;
    theme: Theme;
    books: Book[];
  }
  let { style, theme, books }: Props = $props();

  const SLOTS_PER_ROW = 6;

  // Books in display order — Folio first, then books in vault-supplied order.
  // We layout into rows of SLOTS_PER_ROW with a trailing "+" slot at the
  // end of the last row so create is always discoverable.
  type Item =
    | { kind: "folio" }
    | { kind: "book"; book: Book }
    | { kind: "add" };

  let rows = $derived.by<Item[][]>(() => {
    const items: Item[] = [
      { kind: "folio" },
      ...books.map((b) => ({ kind: "book" as const, book: b })),
      { kind: "add" as const },
    ];
    const out: Item[][] = [];
    const rowCount = Math.max(2, Math.ceil(items.length / SLOTS_PER_ROW));
    for (let i = 0; i < rowCount; i++) {
      out.push(items.slice(i * SLOTS_PER_ROW, (i + 1) * SLOTS_PER_ROW));
    }
    return out;
  });

  // ---- inline create + rename ----
  let creating = $state(false);
  let createName = $state("");
  let createError = $state<string | null>(null);
  let renamingName = $state<string | null>(null);
  let renameValue = $state("");
  let renameError = $state<string | null>(null);

  async function commitCreate() {
    const name = createName.trim();
    if (!name) {
      creating = false;
      createName = "";
      return;
    }
    try {
      await createBook(name);
      creating = false;
      createName = "";
      createError = null;
    } catch (e) {
      createError = String(e);
    }
  }

  async function commitRename() {
    const old = renamingName;
    const next = renameValue.trim();
    if (!old || !next || next === old) {
      renamingName = null;
      return;
    }
    try {
      await renameBook(old, next);
      renamingName = null;
      renameError = null;
    } catch (e) {
      renameError = String(e);
    }
  }

  // ---- context menu ----
  let menu = $state<{ x: number; y: number; items: MenuItem[] } | null>(null);

  function openSpineMenu(ev: MouseEvent, b: Book) {
    ev.preventDefault();
    menu = {
      x: ev.clientX,
      y: ev.clientY,
      items: [
        { label: "Open", action: () => selectBook(b.name) },
        { separator: true, label: "", action: () => {} },
        {
          label: "Rename…",
          action: () => {
            renamingName = b.name;
            renameValue = b.name;
            renameError = null;
          },
        },
        {
          label: "Delete…",
          danger: true,
          action: () => {
            pendingDelete = { name: b.name, pageCount: b.page_count };
          },
        },
      ],
    };
  }

  function openShelfMenu(ev: MouseEvent) {
    ev.preventDefault();
    menu = {
      x: ev.clientX,
      y: ev.clientY,
      items: [
        {
          label: "New book…",
          action: () => {
            creating = true;
            createName = "";
            createError = null;
          },
        },
      ],
    };
  }

  // ---- delete confirmation ----
  let pendingDelete = $state<{ name: string; pageCount: number } | null>(null);
  let deleteError = $state<string | null>(null);
  let deleting = $state(false);

  async function confirmDelete(alsoDeletePages: boolean) {
    if (!pendingDelete) return;
    deleting = true;
    deleteError = null;
    const target = pendingDelete;
    try {
      await deleteBook(target.name, alsoDeletePages);
      pendingDelete = null;
      // If the active book was the one we just removed, fall back to Folio.
      if (vaultState.activeBook === target.name) {
        await selectBook(null);
      }
    } catch (e) {
      deleteError = String(e);
    } finally {
      deleting = false;
    }
  }

  // ---- drag reorder ----
  let dragName = $state<string | null>(null);
  let dragOver = $state<string | null>(null);

  function onDragStart(ev: DragEvent, b: Book) {
    if (!ev.dataTransfer) return;
    dragName = b.name;
    // Allow both the "reorder within the shelf" operation and the
    // "drop onto an editor pane to open the book" operation. The pane
    // drop handler only consumes x-skein-book; reorder is keyed off
    // dragName + dragOver state.
    ev.dataTransfer.effectAllowed = "copyMove";
    ev.dataTransfer.setData("application/x-skein-book", b.name);
  }

  function onDragOver(ev: DragEvent, targetName: string) {
    if (!dragName || dragName === targetName) return;
    ev.preventDefault();
    dragOver = targetName;
    if (ev.dataTransfer) ev.dataTransfer.dropEffect = "move";
  }

  async function onDrop(ev: DragEvent, targetName: string) {
    ev.preventDefault();
    const src = dragName;
    dragName = null;
    dragOver = null;
    if (!src || src === targetName) return;
    const order = books.map((b) => b.name).filter((n) => n !== src);
    const insertAt = order.indexOf(targetName);
    if (insertAt === -1) return;
    order.splice(insertAt, 0, src);
    try {
      await setBookOrder(order);
    } catch (e) {
      console.error(e);
    }
  }

  function onDragEnd() {
    dragName = null;
    dragOver = null;
  }
</script>

<div
  class="sk-shelf style-{style}"
  oncontextmenu={(e) => {
    // Only open the shelf menu when the right-click landed on shelf
    // background (a row gap or empty slot) — not on a child button.
    const t = e.target as HTMLElement;
    if (t.closest("button.bare") || t.closest(".sk-spine")) return;
    openShelfMenu(e);
  }}
>
  {#each rows as row, ri (ri)}
    <div class="sk-shelf-row">
      {#each row as item, ci (ci)}
        {#if item.kind === "folio"}
          <button class="bare" onclick={() => selectBook(null)} aria-label="Open Folio">
            <Folio />
          </button>
        {:else if item.kind === "book"}
          {@const b = item.book}
          {#if renamingName === b.name}
            <div class="rename-slot">
              <!-- svelte-ignore a11y_autofocus -->
              <input
                bind:value={renameValue}
                autofocus
                onkeydown={(e) => {
                  if (e.key === "Enter") void commitRename();
                  if (e.key === "Escape") {
                    renamingName = null;
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
              class="bare"
              class:drag-over={dragOver === b.name}
              draggable="true"
              onclick={() => selectBook(b.name)}
              oncontextmenu={(e) => openSpineMenu(e, b)}
              ondragstart={(e) => onDragStart(e, b)}
              ondragover={(e) => onDragOver(e, b.name)}
              ondrop={(e) => onDrop(e, b.name)}
              ondragend={onDragEnd}
              aria-label={`Open ${b.name}`}
            >
              <Spine
                title={b.name}
                h={spineHeight(b.name)}
                hue={spineHue(b.name)}
                shade={spineShade(b.name)}
                w={spineWidth(b.name)}
                active={vaultState.activeBook === b.name}
                {theme}
              />
            </button>
          {/if}
        {:else if item.kind === "add"}
          {#if creating}
            <div class="rename-slot">
              <!-- svelte-ignore a11y_autofocus -->
              <input
                bind:value={createName}
                autofocus
                placeholder="book name"
                onkeydown={(e) => {
                  if (e.key === "Enter") void commitCreate();
                  if (e.key === "Escape") {
                    creating = false;
                    createName = "";
                    createError = null;
                  }
                }}
                onblur={() => void commitCreate()}
              />
              {#if createError}
                <span class="err">{createError}</span>
              {/if}
            </div>
          {:else}
            <button
              class="bare add-slot"
              onclick={() => {
                creating = true;
                createName = "";
                createError = null;
              }}
              aria-label="New book"
              title="New book"
            >
              +
            </button>
          {/if}
        {/if}
      {/each}
    </div>
  {/each}
  <div style:height="12px"></div>
</div>

{#if menu}
  <ContextMenu x={menu.x} y={menu.y} items={menu.items} onclose={() => (menu = null)} />
{/if}

{#if pendingDelete}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay" onclick={() => (pendingDelete = null)}>
    <div
      class="modal"
      role="dialog"
      aria-label="Delete book"
      onclick={(e) => e.stopPropagation()}
    >
      <h3>Delete "{pendingDelete.name}"?</h3>
      {#if pendingDelete.pageCount > 0}
        <p>
          This book contains {pendingDelete.pageCount}
          {pendingDelete.pageCount === 1 ? "page" : "pages"}. Also delete the pages?
        </p>
        <div class="actions">
          <button onclick={() => (pendingDelete = null)}>Cancel</button>
          <button onclick={() => confirmDelete(false)} disabled={deleting}>
            No, move pages to Folio
          </button>
          <button class="danger" onclick={() => confirmDelete(true)} disabled={deleting}>
            Yes, delete pages
          </button>
        </div>
      {:else}
        <p>This book is empty.</p>
        <div class="actions">
          <button onclick={() => (pendingDelete = null)}>Cancel</button>
          <button class="danger" onclick={() => confirmDelete(true)} disabled={deleting}>
            Delete
          </button>
        </div>
      {/if}
      {#if deleteError}<p class="err">{deleteError}</p>{/if}
    </div>
  </div>
{/if}

<style>
  .bare {
    background: transparent;
    border: 0;
    padding: 0;
    margin: 0;
    cursor: pointer;
    font: inherit;
    color: inherit;
    display: contents;
  }
  .bare.drag-over :global(.sk-spine) {
    transform: translateY(-4px);
    outline: 2px dashed var(--accent-edge, oklch(0.78 0.13 75));
    outline-offset: 2px;
  }
  .add-slot {
    display: flex;
    align-items: flex-end;
    width: 28px;
    height: 76px;
    color: oklch(0.65 0.02 70);
    background: oklch(0 0 0 / 0.12);
    border-radius: 4px;
    border: 1px dashed oklch(0.5 0.012 70);
    justify-content: center;
    padding-bottom: 8px;
    font-size: 18px;
    line-height: 1;
    flex: 0 0 auto;
  }
  .add-slot:hover {
    color: oklch(0.92 0.012 80);
    border-color: var(--accent-edge, oklch(0.78 0.13 75));
  }
  .rename-slot {
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 76px;
    width: 90px;
    flex: 0 0 auto;
    justify-content: flex-end;
  }
  .rename-slot input {
    width: 100%;
    background: var(--page, #20201f);
    color: var(--ink, #e6e6e6);
    border: 1px solid var(--accent-edge, oklch(0.78 0.13 75));
    border-radius: 3px;
    font: inherit;
    font-size: 11px;
    padding: 3px 6px;
    text-align: center;
  }
  .err {
    font-size: 10.5px;
    color: oklch(0.7 0.16 25);
    margin-top: 2px;
    text-align: center;
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
    font-family: "Inter", system-ui, sans-serif;
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
