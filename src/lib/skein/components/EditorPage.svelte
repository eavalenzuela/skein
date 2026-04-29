<script lang="ts">
  import type { Tab } from "../tabs.svelte.js";
  import { setBody } from "../tabs.svelte.js";
  import Editor from "../editor/Editor.svelte";
  import TagChips from "./TagChips.svelte";

  interface Props {
    tab: Tab;
  }
  let { tab }: Props = $props();

  let dropping = $state(false);
  let dragDepth = 0;

  function isChatDrag(types: readonly string[] | DataTransfer["types"]) {
    return Array.from(types ?? []).includes("application/x-skein-chat");
  }

  function onDragEnter(e: DragEvent) {
    if (!e.dataTransfer || !isChatDrag(e.dataTransfer.types)) return;
    dragDepth++;
    dropping = true;
  }
  function onDragLeave() {
    dragDepth = Math.max(0, dragDepth - 1);
    if (dragDepth === 0) dropping = false;
  }
  function onDragEndReset() {
    dragDepth = 0;
    dropping = false;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="sk-page"
  class:dropping
  ondragenter={onDragEnter}
  ondragleave={onDragLeave}
  ondrop={onDragEndReset}
>
  {#if tab.loading}
    <div class="loading">Loading…</div>
  {:else}
    <TagChips {tab} />
    <Editor
      doc={tab.body}
      relPath={tab.rel_path}
      onChange={(next) => setBody(tab.rel_path, next)}
    />
  {/if}
  {#if dropping}
    <div class="drop-hint" aria-hidden="true">Drop to insert into the page</div>
  {/if}
</div>

<style>
  .sk-page :global(.cm-editor) {
    background: var(--page);
  }
  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--ink-3);
    font-family: "Inter", sans-serif;
    font-size: 12px;
  }
  .sk-page.dropping {
    box-shadow: inset 0 0 0 2px var(--accent);
  }
  .drop-hint {
    position: absolute;
    top: 12px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--accent);
    color: oklch(0.98 0.01 60);
    font-family: "Inter", system-ui, sans-serif;
    font-size: 11.5px;
    padding: 4px 12px;
    border-radius: 12px;
    pointer-events: none;
    box-shadow: 0 4px 12px oklch(0 0 0 / 0.3);
  }
</style>
