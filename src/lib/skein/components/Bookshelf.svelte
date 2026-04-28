<script lang="ts">
  import type { ShelfStyle, Theme } from "../tweaks.svelte.js";
  import { BOOKS } from "../data.js";
  import Spine from "./Spine.svelte";
  import Folio from "./Folio.svelte";

  interface Props {
    style: ShelfStyle;
    theme: Theme;
  }
  let { style, theme }: Props = $props();

  let row1 = $derived(BOOKS.slice(0, 6));
  let row2 = $derived(BOOKS.slice(6));
</script>

<div class="sk-shelf style-{style}">
  <div class="sk-shelf-row">
    {#each row1 as book, i (i)}
      {#if book.kind === "folio"}
        <Folio />
      {:else}
        <Spine
          title={book.title}
          h={book.h}
          hue={book.hue}
          shade={book.shade}
          active={book.active}
          {theme}
        />
      {/if}
    {/each}
  </div>
  <div class="sk-shelf-row">
    {#each row2 as book, i (i + 100)}
      {#if book.kind === "folio"}
        <Folio />
      {:else}
        <Spine
          title={book.title}
          h={book.h}
          hue={book.hue}
          shade={book.shade}
          active={book.active}
          {theme}
        />
      {/if}
    {/each}
  </div>
  <div style:height="12px"></div>
</div>
