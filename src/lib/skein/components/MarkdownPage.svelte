<script lang="ts">
  import type { OpenPage } from "../vault.svelte.js";

  interface Props {
    page: OpenPage;
  }
  let { page }: Props = $props();

  // Strip YAML frontmatter for the read-only Phase 2 preview.
  // Phase 3 replaces this with CodeMirror + live preview.
  let body = $derived.by(() => {
    const m = page.body.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
    return m ? m[2] : page.body;
  });
</script>

<div class="sk-page-inner phase2-preview">
  <h1>{page.title}</h1>
  <div class="muted">{page.rel_path}</div>
  <pre class="raw">{body}</pre>
</div>

<style>
  .phase2-preview pre.raw {
    font-family: "JetBrains Mono", monospace;
    font-size: 12.5px;
    line-height: 1.55;
    color: var(--ink-2);
    background: oklch(from var(--page) calc(l - 0.02) c h);
    padding: 12px 14px;
    border-radius: 4px;
    margin-top: 18px;
    white-space: pre-wrap;
    word-wrap: break-word;
    max-height: calc(100vh - 280px);
    overflow: auto;
  }
</style>
