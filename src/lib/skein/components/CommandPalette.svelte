<script lang="ts">
  import {
    searchPages,
    type SearchHit,
    openTodayDaily,
    rebuildIndex,
    createPage,
  } from "../vault.js";
  import { openTab } from "../tabs.svelte.js";
  import { closeSearch } from "../searchUi.svelte.js";
  import { openSettings } from "../settingsUi.svelte.js";
  import { close as closeVaultStore, vaultState } from "../vault.svelte.js";
  import { downloadModel } from "../embedder.svelte.js";
  import { focusTrap } from "../focusTrap.js";

  interface Command {
    id: string;
    label: string;
    keywords: string;
    hint?: string;
    run: () => Promise<void> | void;
  }

  const COMMANDS: Command[] = [
    {
      id: "settings",
      label: "Open settings",
      keywords: "settings preferences config",
      hint: "Ctrl+,",
      run: () => {
        closeSearch();
        openSettings();
      },
    },
    {
      id: "today",
      label: "Open today's daily note",
      keywords: "daily today journal",
      run: async () => {
        const res = await openTodayDaily();
        const stem = res.rel_path.split("/").pop()?.replace(/\.md$/, "") ?? res.rel_path;
        await openTab({ rel_path: res.rel_path, title: stem });
        closeSearch();
      },
    },
    {
      id: "new-page-folio",
      label: "New page in Folio (loose)…",
      keywords: "new page create folio loose",
      run: async () => {
        const title = window.prompt("Title for the new page:");
        if (!title || !title.trim()) return;
        const rel = await createPage(null, title.trim());
        await openTab({ rel_path: rel, title: title.trim() });
        closeSearch();
      },
    },
    {
      id: "switch-vault",
      label: "Switch vault — close this one",
      keywords: "vault switch close picker",
      run: async () => {
        await closeVaultStore();
        closeSearch();
      },
    },
    {
      id: "rebuild-index",
      label: "Rebuild search index",
      keywords: "rebuild reindex search index",
      run: async () => {
        await rebuildIndex();
        closeSearch();
      },
    },
    {
      id: "download-model",
      label: "Download local embedding model (~130 MB)",
      keywords: "download embedder bge model semantic",
      run: async () => {
        closeSearch();
        openSettings();
        await downloadModel();
      },
    },
  ];

  let query = $state("");
  let hits = $state<SearchHit[]>([]);
  let cmdHits = $state<Command[]>([]);
  let active = $state(0);
  let loading = $state(false);
  let lastQuery = "";
  let pending = 0;
  let inputEl: HTMLInputElement | undefined = $state();

  let isCommandMode = $derived(query.startsWith(":"));
  let resultCount = $derived(isCommandMode ? cmdHits.length : hits.length);

  $effect(() => {
    inputEl?.focus();
  });

  function filterCommands(q: string): Command[] {
    const term = q.replace(/^:/, "").trim().toLowerCase();
    if (!term) return COMMANDS;
    return COMMANDS.filter(
      (c) =>
        c.label.toLowerCase().includes(term) ||
        c.keywords.toLowerCase().includes(term),
    );
  }

  async function runSearch(q: string) {
    if (q.trim() === "") {
      hits = [];
      cmdHits = [];
      active = 0;
      return;
    }
    if (q.startsWith(":")) {
      cmdHits = filterCommands(q);
      hits = [];
      active = 0;
      loading = false;
      return;
    }
    const myToken = ++pending;
    loading = true;
    try {
      const result = await searchPages(q, 30);
      if (myToken !== pending) return;
      hits = result;
      cmdHits = [];
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

  async function selectCommand(cmd: Command) {
    try {
      await cmd.run();
    } catch (e) {
      console.error("command failed", cmd.id, e);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      closeSearch();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (resultCount > 0) active = (active + 1) % resultCount;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (resultCount > 0) active = (active - 1 + resultCount) % resultCount;
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (isCommandMode) {
        const cmd = cmdHits[active];
        if (cmd) void selectCommand(cmd);
      } else {
        const hit = hits[active];
        if (hit) selectHit(hit);
      }
    }
  }

  // Suppress an unused-import warning on builds that tree-shake.
  void vaultState;
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="palette-overlay"
  onclick={closeSearch}
  onkeydown={(e) => e.key === "Escape" && closeSearch()}
>
  <div
    class="palette"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-label="Search pages and run commands"
    aria-modal="true"
    tabindex="-1"
    use:focusTrap
  >
    <input
      bind:this={inputEl}
      bind:value={query}
      type="text"
      placeholder="Search pages — start with : for commands"
      onkeydown={onKeydown}
      autocomplete="off"
      spellcheck="false"
      aria-label="Search pages or commands"
    />
    <div class="hint-row">
      <span
        >{loading
          ? "searching…"
          : isCommandMode
            ? cmdHits.length
              ? `${cmdHits.length} command${cmdHits.length === 1 ? "" : "s"}`
              : "no matching commands"
            : hits.length
              ? `${hits.length} match${hits.length === 1 ? "" : "es"}`
              : query.trim()
                ? "no matches"
                : "type to search · : for commands"}</span
      >
      <span class="kbd">↑↓ navigate · ⏎ open · esc close</span>
    </div>
    <ul class="results">
      {#if isCommandMode}
        {#each cmdHits as cmd, i (cmd.id)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li
            class:active={i === active}
            onclick={() => selectCommand(cmd)}
            onmouseenter={() => (active = i)}
            role="option"
            aria-selected={i === active}
          >
            <div class="ttl cmd-row">
              <span class="cmd-label">{cmd.label}</span>
              {#if cmd.hint}<span class="cmd-hint">{cmd.hint}</span>{/if}
            </div>
          </li>
        {/each}
      {:else}
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
      {/if}
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
  .cmd-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .cmd-hint {
    font-family: "JetBrains Mono", monospace;
    font-size: 10.5px;
    color: var(--ink-4);
    border: 1px solid var(--chrome-edge);
    border-radius: 3px;
    padding: 1px 5px;
  }
</style>
