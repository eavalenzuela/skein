<script lang="ts">
  import type { Page } from "../vault.js";
  import { openPageByPath } from "../vault.svelte.js";

  interface Props {
    pages: Page[];
    title: string;
  }
  let { pages, title }: Props = $props();
</script>

<div class="page-list">
  <div class="hdr">
    <h2>{title}</h2>
    <span class="count">{pages.length} {pages.length === 1 ? "page" : "pages"}</span>
  </div>
  {#if pages.length === 0}
    <p class="empty">No pages yet.</p>
  {:else}
    <ul>
      {#each pages as p (p.rel_path)}
        <li>
          <button onclick={() => openPageByPath(p.rel_path, p.title)}>
            <span class="ttl">{p.title}</span>
            <span class="rp">{p.rel_path}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

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
  .empty {
    color: var(--ink-3);
    font-size: 12px;
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
</style>
