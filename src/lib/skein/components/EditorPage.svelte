<script lang="ts">
  import type { Tab } from "../tabs.svelte.js";
  import { setBody } from "../tabs.svelte.js";
  import { docStats } from "../wordCount.js";
  import Editor from "../editor/Editor.svelte";
  import TagChips from "./TagChips.svelte";

  interface Props {
    tab: Tab;
  }
  let { tab }: Props = $props();

  let stats = $derived(docStats(tab.body));

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
    <div class="doc-stats" aria-label="Document statistics">
      {stats.words}
      {stats.words === 1 ? "word" : "words"} · {stats.chars} chars{stats.minutes > 0
        ? ` · ~${stats.minutes} min`
        : ""}
    </div>
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
  .doc-stats {
    flex: 0 0 auto;
    padding: 3px 14px 5px;
    text-align: right;
    color: var(--ink-4);
    font-family: "JetBrains Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.03em;
    border-top: 1px solid var(--page-edge);
    background: var(--page);
    user-select: none;
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
