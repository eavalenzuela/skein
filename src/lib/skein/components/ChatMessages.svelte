<script lang="ts">
  import { chatState } from "../chat.svelte.js";

  function onDragStart(e: DragEvent) {
    const sel = window.getSelection();
    if (!sel || sel.toString().length === 0) return;
    e.dataTransfer?.setData("text/plain", sel.toString());
    e.dataTransfer?.setData("application/x-skein-chat", sel.toString());
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "copy";
  }
</script>

<div class="sk-chat" role="log" aria-live="polite">
  {#each chatState.messages as m (m.id)}
    {#if m.role === "user"}
      <div
        class="sk-msg user"
        role="article"
        draggable="true"
        ondragstart={onDragStart}
        title="Drag a selection into the editor to insert"
      >
        {m.content}
      </div>
    {:else}
      <div
        class="sk-msg asst"
        role="article"
        draggable="true"
        ondragstart={onDragStart}
        title="Drag a selection into the editor to insert"
      >
        <div class="who">
          <span class="ico">A</span>
          {chatState.model.replace("claude-", "")} · {m.contextMode ?? chatState.contextMode}
        </div>
        <div class="body">
          {m.content}{#if m.streaming}<span class="streaming-dot"></span>{/if}
        </div>
        {#if m.contextChunks && m.contextChunks.length > 0}
          <div class="ctx">
            <span class="ctx-label">context:</span>
            {#each m.contextChunks as c, i (i)}
              <span class="ctx-chip" title={c.heading || c.rel_path}>
                {c.title}{#if c.heading}<span class="ctx-h">·{c.heading}</span>{/if}
              </span>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  {/each}

  {#if chatState.error}
    <div class="error">{chatState.error}</div>
  {/if}
</div>

<style>
  /* Most chat styling lives in styles.css to inherit the design's tokens.
     Local styles cover the additions Phase 7 introduces. */
  .sk-msg.asst .body {
    white-space: pre-wrap;
  }
  .sk-msg[draggable="true"] {
    cursor: grab;
  }
  .sk-msg[draggable="true"]:active {
    cursor: grabbing;
  }
  .ctx {
    margin-top: 6px;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    align-items: center;
    font-size: 10.5px;
  }
  .ctx-label {
    color: var(--ink-4);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .ctx-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 6px;
    border: 1px solid var(--chrome-edge);
    border-radius: 8px;
    color: var(--ink-3);
    background: oklch(from var(--chrome-2) calc(l + 0.02) c h);
    font-size: 10px;
    cursor: default;
  }
  .ctx-h {
    color: var(--ink-4);
  }
  .error {
    margin-top: 8px;
    color: oklch(0.65 0.18 25);
    font-size: 11px;
    border-left: 2px solid oklch(0.65 0.18 25 / 0.5);
    padding-left: 8px;
  }
</style>
