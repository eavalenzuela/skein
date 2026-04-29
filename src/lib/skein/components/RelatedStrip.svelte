<script lang="ts">
  import { findRelated, type RelatedHit } from "../vault.js";
  import { openTab, activeTab } from "../tabs.svelte.js";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy } from "svelte";

  let hits = $state<RelatedHit[]>([]);
  let loading = $state(false);
  let lastPath = "";
  let unlisten: UnlistenFn | null = null;

  async function refresh(relPath: string | null) {
    if (!relPath) {
      hits = [];
      return;
    }
    loading = true;
    try {
      hits = await findRelated(relPath, 6);
    } catch {
      hits = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    const a = activeTab();
    const path = a?.rel_path ?? null;
    if (path === lastPath) return;
    lastPath = path ?? "";
    refresh(path);
  });

  // Re-fetch when the indexer notifies us.
  (async () => {
    unlisten = await listen("vault-changed", () => {
      const a = activeTab();
      if (a) refresh(a.rel_path);
    });
  })();

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  function pct(sim: number): string {
    return `${Math.round(sim * 100)}`;
  }

  function onPillKey(e: KeyboardEvent, idx: number) {
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
      e.preventDefault();
      const dir = e.key === "ArrowRight" ? 1 : -1;
      const next = (idx + dir + hits.length) % hits.length;
      const el = document.querySelector<HTMLButtonElement>(`[data-rel-pill="${next}"]`);
      el?.focus();
    } else if (e.key === "Home") {
      e.preventDefault();
      document.querySelector<HTMLButtonElement>(`[data-rel-pill="0"]`)?.focus();
    } else if (e.key === "End") {
      e.preventDefault();
      document
        .querySelector<HTMLButtonElement>(`[data-rel-pill="${hits.length - 1}"]`)
        ?.focus();
    }
  }
</script>

{#if hits.length > 0 || loading}
  <div class="related-strip">
    <div class="label">Related</div>
    <div class="hits">
      {#if loading && hits.length === 0}
        <span class="empty">finding…</span>
      {:else}
        {#each hits as hit, i (hit.rel_path)}
          <button
            class="pill"
            title={hit.snippet}
            onclick={() => openTab({ rel_path: hit.rel_path, title: hit.title })}
            onkeydown={(e) => onPillKey(e, i)}
            data-rel-pill={i}
          >
            <span class="ttl">{hit.title}</span>
            <span class="sim">{pct(hit.similarity)}</span>
          </button>
        {/each}
      {/if}
    </div>
  </div>
{/if}

<style>
  .related-strip {
    position: relative;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 18px;
    background: var(--paper-2);
    border-top: 1px solid var(--chrome-edge);
    font-family: "Inter", system-ui, sans-serif;
    flex-shrink: 0;
    min-height: 44px;
  }
  .label {
    font-size: 10.5px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-4);
    flex-shrink: 0;
  }
  .hits {
    display: flex;
    flex-wrap: nowrap;
    gap: 6px;
    overflow-x: auto;
    overflow-y: hidden;
    flex: 1;
    min-width: 0;
  }
  .empty {
    color: var(--ink-4);
    font-size: 11.5px;
    font-style: italic;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 24px;
    padding: 0 10px;
    border: 1px solid var(--chrome-edge);
    border-radius: 12px;
    background: var(--page);
    color: var(--ink-2);
    font-family: "Source Serif 4", Georgia, serif;
    font-size: 12px;
    cursor: pointer;
    flex-shrink: 0;
    max-width: 220px;
  }
  .pill:hover {
    background: oklch(from var(--page) calc(l + 0.02) c h);
    color: var(--ink);
    border-color: var(--accent-edge);
  }
  .pill .ttl {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .pill .sim {
    font-family: "JetBrains Mono", monospace;
    font-size: 10px;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 1px 5px;
    border-radius: 8px;
    flex-shrink: 0;
  }
</style>
