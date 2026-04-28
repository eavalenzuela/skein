<script lang="ts">
  import type { Book } from "../vault.js";
  import type { ShelfStyle, Theme } from "../tweaks.svelte.js";
  import { spineHeight, spineHue, spineShade } from "../spineHash.js";
  import { vaultState, selectBook } from "../vault.svelte.js";
  import Spine from "./Spine.svelte";
  import Folio from "./Folio.svelte";

  interface Props {
    style: ShelfStyle;
    theme: Theme;
    books: Book[];
  }
  let { style, theme, books }: Props = $props();

  // 6 slots per row matches the mockup.
  const SLOTS_PER_ROW = 6;

  // Folio + books, then split into rows.
  let rows = $derived.by<{ kind: "folio" | "book"; book?: Book }[][]>(() => {
    const items: { kind: "folio" | "book"; book?: Book }[] = [
      { kind: "folio" },
      ...books.map((b) => ({ kind: "book" as const, book: b })),
    ];
    const out: { kind: "folio" | "book"; book?: Book }[][] = [];
    let rowCount = Math.max(2, Math.ceil(items.length / SLOTS_PER_ROW));
    for (let i = 0; i < rowCount; i++) {
      out.push(items.slice(i * SLOTS_PER_ROW, (i + 1) * SLOTS_PER_ROW));
    }
    return out;
  });
</script>

<div class="sk-shelf style-{style}">
  {#each rows as row, ri (ri)}
    <div class="sk-shelf-row">
      {#each row as item, ci (ci)}
        {#if item.kind === "folio"}
          <button class="bare" onclick={() => selectBook(null)} aria-label="Open Folio">
            <Folio />
          </button>
        {:else if item.book}
          {@const b = item.book}
          <button class="bare" onclick={() => selectBook(b.name)} aria-label={`Open ${b.name}`}>
            <Spine
              title={b.name}
              h={spineHeight(b.name)}
              hue={spineHue(b.name)}
              shade={spineShade(b.name)}
              active={vaultState.activeBook === b.name}
              {theme}
            />
          </button>
        {/if}
      {/each}
    </div>
  {/each}
  <div style:height="12px"></div>
</div>

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
</style>
