<script lang="ts">
  import { searchPages, type SearchHit } from "../vault.js";
  import { openTab } from "../tabs.svelte.js";
  import { closeSearch } from "../searchUi.svelte.js";

  let query = $state("");
  let hits = $state<SearchHit[]>([]);
  let active = $state(0);
  let loading = $state(false);
  let lastQuery = "";
  let pending = 0;
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    inputEl?.focus();
  });

  async function runSearch(q: string) {
    if (q.trim() === "") {
      hits = [];
      active = 0;
      return;
    }
    const myToken = ++pending;
    loading = true;
    try {
      const result = await searchPages(q, 30);
      if (myToken !== pending) return; // a newer query started
      hits = result;
      active = 0;
    } catch {
      if (myToken === pending) hits = [];
    } finally {
      if (myToken === pending) loading = false;
    }
  }

  $effect(() => {
    const q = query;
    if (q === lastQuery) return;
    lastQuery = q;
    const handle = setTimeout(() => runSearch(q), 80);
    return () => clearTimeout(handle);
  });

  function selectHit(hit: SearchHit) {
    openTab({ rel_path: hit.rel_path, title: hit.title });
    closeSearch();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      closeSearch();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (hits.length > 0) active = (active + 1) % hits.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (hits.length > 0) active = (active - 1 + hits.length) % hits.length;
    } else if (e.key === "Enter") {
      e.preventDefault();
      const hit = hits[active];
      if (hit) selectHit(hit);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="palette-overlay" onclick={closeSearch}>
  <div
    class="palette"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-label="Search pages"
    tabindex="-1"
  >
    <input
      bind:this={inputEl}
      bind:value={query}
      type="text"
      placeholder="Search pages…"
      onkeydown={onKeydown}
      autocomplete="off"
      spellcheck="false"
    />
    <div class="hint-row">
      <span
        >{loading
          ? "searching…"
          : hits.length
            ? `${hits.length} match${hits.length === 1 ? "" : "es"}`
            : query.trim()
              ? "no matches"
              : "type to search"}</span
      >
      <span class="kbd">↑↓ navigate · ⏎ open · esc close</span>
    </div>
    <ul class="results">
      {#each hits as hit, i (hit.rel_path)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <li
          class:active={i === active}
          onclick={() => selectHit(hit)}
          onmouseenter={() => (active = i)}
          role="option"
          aria-selected={i === active}
        >
          <div class="ttl">{hit.title}</div>
          <div class="rp">{hit.book ?? "Folio"} · {hit.rel_path}</div>
          <!-- snippet contains <mark> tags from FTS5 -->
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="snip">{@html hit.snippet}</div>
        </li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    background: oklch(0 0 0 / 0.65);
    z-index: 800;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 12vh;
    backdrop-filter: blur(3px);
  }
  .palette {
    width: 640px;
    max-width: 92vw;
    background: var(--chrome-2);
    border: 1px solid var(--chrome-edge);
    border-radius: 10px;
    overflow: hidden;
    box-shadow: 0 30px 80px -20px oklch(0 0 0 / 0.55);
    color: var(--ink);
    font-family: "Inter", system-ui, sans-serif;
  }
  input {
    width: 100%;
    padding: 14px 18px;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--chrome-edge);
    color: var(--ink);
    font-family: "Inter", system-ui, sans-serif;
    font-size: 15px;
    outline: none;
  }
  input::placeholder {
    color: var(--ink-4);
  }
  .hint-row {
    display: flex;
    justify-content: space-between;
    padding: 6px 18px;
    font-size: 11px;
    color: var(--ink-4);
    border-bottom: 1px solid var(--chrome-edge);
    letter-spacing: 0.04em;
  }
  .kbd {
    font-family: "JetBrains Mono", monospace;
  }
  .results {
    list-style: none;
    margin: 0;
    padding: 6px 0;
    max-height: 50vh;
    overflow: auto;
  }
  li {
    padding: 8px 18px;
    cursor: pointer;
    border-left: 2px solid transparent;
  }
  li.active {
    background: oklch(from var(--chrome-2) calc(l + 0.04) c h);
    border-left-color: var(--accent);
  }
  .ttl {
    font-family: "Source Serif 4", Georgia, serif;
    font-size: 14px;
    color: var(--ink);
    margin-bottom: 2px;
  }
  .rp {
    font-family: "JetBrains Mono", monospace;
    font-size: 10.5px;
    color: var(--ink-4);
    margin-bottom: 4px;
  }
  .snip {
    font-size: 12px;
    color: var(--ink-3);
    line-height: 1.5;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
  }
  .snip :global(mark) {
    background: var(--accent-soft);
    color: var(--accent);
    padding: 0 2px;
    border-radius: 2px;
  }
</style>
