<script lang="ts">
  import type { Tab } from "../tabs.svelte.js";
  import { setBody } from "../tabs.svelte.js";
  import Editor from "../editor/Editor.svelte";
  import TagChips from "./TagChips.svelte";

  interface Props {
    tab: Tab;
  }
  let { tab }: Props = $props();
</script>

<div class="sk-page">
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
</style>
