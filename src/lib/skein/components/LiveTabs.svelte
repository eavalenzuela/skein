<script lang="ts">
  import type { Tab } from "../tabs.svelte.js";
  import { isDirty, cyclePin, unpin } from "../tabs.svelte.js";

  interface Props {
    tabs: Tab[];
    activeId: string | null;
    onSelect: (relPath: string) => void;
    onClose: (relPath: string) => void;
  }
  let { tabs, activeId, onSelect, onClose }: Props = $props();

  function pinClass(pin: Tab["pin"]): string {
    if (pin === "left") return "pinned-l";
    if (pin === "right") return "pinned-r";
    return "";
  }
</script>

<div class="sk-tabs">
  {#each tabs as tab (tab.rel_path)}
    <div class="sk-tab {tab.rel_path === activeId ? 'active' : ''} {pinClass(tab.pin)}">
      <button
        class="bare pin"
        title={tab.pin === "left"
          ? "Pinned left · click to move to right · right-click to unpin"
          : tab.pin === "right"
            ? "Pinned right · click to unpin · right-click to unpin"
            : "Pin tab — click to dock for split view (cycles left → right → none)"}
        onclick={(e) => {
          e.stopPropagation();
          cyclePin(tab.rel_path);
        }}
        oncontextmenu={(e) => {
          e.preventDefault();
          if (tab.pin) unpin(tab.rel_path);
        }}
        aria-label="Pin tab"
      >
        <span class="pin-ind">
          {#if tab.pin === "left"}
            <svg
              width="10"
              height="10"
              viewBox="0 0 12 12"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
            >
              <path d="M2 2v8M5 4l4-2v8L5 8z" fill="currentColor" fill-opacity="0.4" />
            </svg>
          {:else if tab.pin === "right"}
            <svg
              width="10"
              height="10"
              viewBox="0 0 12 12"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
            >
              <path d="M10 2v8M7 4l-4-2v8l4-2z" fill="currentColor" fill-opacity="0.4" />
            </svg>
          {:else}
            <svg
              width="10"
              height="10"
              viewBox="0 0 12 12"
              fill="none"
              stroke="currentColor"
              stroke-width="1.3"
            >
              <path d="M2.5 1.5h5l2 2v7h-7zM7 1.5v2h2" />
            </svg>
          {/if}
        </span>
      </button>
      <button class="bare name-btn" onclick={() => onSelect(tab.rel_path)}>
        <span class="name">{tab.title}</span>
      </button>
      {#if isDirty(tab)}<span class="dirty" title="Unsaved changes"></span>{/if}
      <button
        class="bare x"
        onclick={(e) => {
          e.stopPropagation();
          onClose(tab.rel_path);
        }}
        aria-label="Close tab"
      >
        ×
      </button>
    </div>
  {/each}
</div>

<style>
  .bare {
    background: transparent;
    border: 0;
    padding: 0;
    margin: 0;
    color: inherit;
    font: inherit;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
  }
  .pin {
    height: 100%;
    padding-right: 2px;
  }
  .name-btn {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    height: 100%;
  }
  .name-btn .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .x {
    width: 18px;
    height: 18px;
    border-radius: 3px;
    color: var(--ink-4);
    font-size: 13px;
    line-height: 1;
    margin-left: 2px;
    justify-content: center;
  }
  .x:hover {
    background: oklch(1 0 0 / 0.06);
    color: var(--ink-2);
  }
</style>
