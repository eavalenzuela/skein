<script lang="ts">
  import { onDestroy } from "svelte";
  import { findBacklinks, type BacklinkHit } from "../vault.js";
  import { openTab, activeTab } from "../tabs.svelte.js";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  let hits = $state<BacklinkHit[]>([]);
  let lastPath = "";
  let unlisten: UnlistenFn | null = null;

  async function refresh(relPath: string | null) {
    if (!relPath) {
      hits = [];
      return;
    }
    try {
      hits = await findBacklinks(relPath);
    } catch {
      hits = [];
    }
  }

  $effect(() => {
    const a = activeTab();
    const path = a?.rel_path ?? null;
    if (path === lastPath) return;
    lastPath = path ?? "";
    void refresh(path);
  });

  (async () => {
    unlisten = await listen("vault-changed", () => {
      const a = activeTab();
      if (a) void refresh(a.rel_path);
    });
  })();

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>

{#if hits.length > 0}
  <div class="linked-strip">
    <div class="label">Linked from</div>
    <div class="hits">
      {#each hits as hit (hit.from_rel_path)}
        <button
          class="pill"
          title={hit.alias ? `aliased as "${hit.alias}"` : hit.from_rel_path}
          onclick={() => openTab({ rel_path: hit.from_rel_path, title: hit.from_title })}
        >
          <span class="ttl">{hit.from_title}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .linked-strip {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 18px;
    background: var(--paper-2);
    border-top: 1px solid var(--chrome-edge);
    font-family: "Inter", system-ui, sans-serif;
    flex-shrink: 0;
    min-height: 36px;
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
  .pill {
    display: inline-flex;
    align-items: center;
    height: 24px;
    padding: 0 10px;
    border: 1px dashed var(--accent-edge);
    border-radius: 12px;
    background: var(--page);
    color: var(--accent);
    font-family: "Source Serif 4", Georgia, serif;
    font-size: 12px;
    cursor: pointer;
    flex-shrink: 0;
    max-width: 220px;
  }
  .pill:hover {
    background: var(--accent-soft);
  }
  .pill .ttl {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
