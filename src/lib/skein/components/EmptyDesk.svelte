<script lang="ts">
  import { LOOSE_PAGES } from "../data.js";
</script>

<div class="sk-empty-stage">
  <div class="sk-empty-hint">
    <b>The desk is clear.</b>
    Open a book from the shelf, or pick a loose page below.
  </div>
  {#each LOOSE_PAGES as p, i (i)}
    <div
      class="sk-card"
      style:left={`${p.x}px`}
      style:top={`${p.y}px`}
      style:transform={`rotate(${p.r}deg)`}
    >
      <div class="ttl">{p.ttl}</div>
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      <div class="body">{@html p.body}</div>
      <div class="meta"><span>{p.meta[0]}</span><span>{p.meta[1]}</span></div>
    </div>
  {/each}
</div>
