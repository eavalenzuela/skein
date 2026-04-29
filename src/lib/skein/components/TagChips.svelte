<script lang="ts">
  import { onDestroy } from "svelte";
  import type { Tab } from "../tabs.svelte.js";
  import { suggestTags, applyTag, dismissTag } from "../vault.js";
  import { hasSecret } from "../settings.js";

  interface Props {
    tab: Tab;
  }
  let { tab }: Props = $props();

  let suggestions = $state<string[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let keyConfigured = $state<boolean | null>(null);
  let lastBodyForPath = "";

  const SUGGEST_DEBOUNCE_MS = 3000;
  let timer: ReturnType<typeof setTimeout> | null = null;

  function clearTimer() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  async function ensureKey() {
    if (keyConfigured !== null) return keyConfigured;
    keyConfigured = await hasSecret("anthropic_api_key").catch(() => false);
    return keyConfigured;
  }

  async function runSuggest() {
    if (!tab) return;
    if (!(await ensureKey())) return;
    if (tab.body.trim().length < 40) return; // not worth a call
    error = null;
    loading = true;
    try {
      const res = await suggestTags(tab.rel_path);
      // Filter out tags already present (parse them from the body's frontmatter quickly).
      const existing = parseExistingTags(tab.body);
      suggestions = res.filter((t) => !existing.has(t));
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function parseExistingTags(body: string): Set<string> {
    const m = body.match(/^---\r?\n([\s\S]*?)\r?\n---/);
    if (!m) return new Set();
    const fm = m[1];
    // Match either `tags: [a, b]` or block-form `tags:\n  - a\n  - b`
    const inline = fm.match(/^tags:\s*\[([^\]]*)\]/m);
    if (inline) {
      return new Set(
        inline[1]
          .split(",")
          .map((s) => s.trim().replace(/^["']|["']$/g, ""))
          .filter(Boolean),
      );
    }
    const block = fm.match(/^tags:\s*\n((?:\s*-\s*.+\n?)+)/m);
    if (block) {
      return new Set(
        block[1]
          .split("\n")
          .map((l) =>
            l
              .replace(/^\s*-\s*/, "")
              .trim()
              .replace(/^["']|["']$/g, ""),
          )
          .filter(Boolean),
      );
    }
    return new Set();
  }

  // Trigger 3s after the last edit on this tab (or when switching to a new tab).
  // Touching `tab.body` here registers the dependency so the effect reruns on edits.
  $effect(() => {
    const path = tab.rel_path;
    void tab.body;
    if (path !== lastBodyForPath) {
      suggestions = [];
      error = null;
    }
    lastBodyForPath = path;
    clearTimer();
    timer = setTimeout(runSuggest, SUGGEST_DEBOUNCE_MS);
  });

  onDestroy(clearTimer);

  async function accept(tag: string) {
    suggestions = suggestions.filter((s) => s !== tag);
    try {
      await applyTag(tab.rel_path, tag);
      // tab.body refresh comes from the watcher reconcile loop.
    } catch (e) {
      error = String(e);
    }
  }

  async function dismiss(tag: string) {
    const before = suggestions;
    suggestions = suggestions.filter((s) => s !== tag);
    try {
      await dismissTag(tab.rel_path, tag);
    } catch (e) {
      // Backend rejected the dismissal — the suggestion is still live and
      // would reappear on reload. Restore the chip and surface the error.
      suggestions = before;
      error = `Couldn't dismiss "${tag}": ${String(e)}`;
    }
  }
</script>

{#if keyConfigured && (loading || suggestions.length > 0 || error)}
  <div class="tag-chips" role="region" aria-label="Tag suggestions">
    <span class="label">Suggested tags</span>
    {#if loading && suggestions.length === 0}
      <span class="muted">thinking…</span>
    {:else}
      {#each suggestions as tag (tag)}
        <span class="chip">
          <button class="accept" onclick={() => accept(tag)} title="Add to frontmatter">
            #{tag}
          </button>
          <button class="dismiss" onclick={() => dismiss(tag)} aria-label="Dismiss">×</button>
        </span>
      {/each}
    {/if}
    {#if error}
      <span class="error">{error}</span>
    {/if}
  </div>
{/if}

<style>
  .tag-chips {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 56px;
    border-bottom: 1px solid var(--page-edge);
    font-family: "Inter", system-ui, sans-serif;
    background: oklch(from var(--page) calc(l - 0.01) c h);
  }
  .label {
    font-size: 10.5px;
    color: var(--ink-4);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    margin-right: 4px;
  }
  .muted {
    color: var(--ink-4);
    font-size: 11.5px;
    font-style: italic;
  }
  .chip {
    display: inline-flex;
    align-items: stretch;
    border: 1px solid var(--accent-edge);
    border-radius: 12px;
    overflow: hidden;
    background: var(--accent-soft);
  }
  .accept {
    background: transparent;
    border: 0;
    padding: 2px 10px;
    color: var(--accent);
    font: inherit;
    font-family: "JetBrains Mono", monospace;
    font-size: 11.5px;
    cursor: pointer;
  }
  .accept:hover {
    background: oklch(from var(--accent) l c h / 0.22);
  }
  .dismiss {
    background: transparent;
    border: 0;
    border-left: 1px solid var(--accent-edge);
    padding: 2px 7px;
    color: var(--ink-3);
    font: inherit;
    font-size: 12px;
    line-height: 1;
    cursor: pointer;
  }
  .dismiss:hover {
    background: oklch(from var(--accent) l c h / 0.18);
    color: var(--ink);
  }
  .error {
    color: oklch(0.65 0.18 25);
    font-size: 11px;
    margin-left: 6px;
  }
</style>
