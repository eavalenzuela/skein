<script lang="ts">
  import {
    tweaks,
    type Theme,
    type ShelfStyle,
    type SidebarMode,
    type Scenario,
    type PageFont,
  } from "./tweaks.svelte.js";
  import { embedderState, downloadModel } from "./embedder.svelte.js";

  let open = $state(true);

  const themes: Theme[] = ["dark", "light"];
  const shelves: ShelfStyle[] = ["abstract", "suggestive", "tactile"];
  const sidebars: SidebarMode[] = ["open", "collapsed", "hidden"];
  const scenarios: Scenario[] = ["populated", "dragging", "empty"];
  const fonts: PageFont[] = ["Source Serif 4", "Iowan Old Style", "Spectral", "Lora"];
</script>

<div class="dev-panel" class:closed={!open}>
  <button class="toggle" onclick={() => (open = !open)} aria-label="Toggle dev controls">
    {open ? "×" : "⚙"}
  </button>
  {#if open}
    <h4>Dev controls</h4>
    <p class="hint">
      Mockup tweaks (Phase 1 only). Phase 6 replaces these with persisted user settings.
    </p>

    <fieldset>
      <legend>Theme</legend>
      {#each themes as t (t)}
        <label>
          <input type="radio" bind:group={tweaks.theme} value={t} />
          {t}
        </label>
      {/each}
    </fieldset>

    <fieldset>
      <legend>Shelf realism</legend>
      {#each shelves as s (s)}
        <label>
          <input type="radio" bind:group={tweaks.shelfStyle} value={s} />
          {s}
        </label>
      {/each}
    </fieldset>

    <fieldset>
      <legend>Sidebar</legend>
      {#each sidebars as s (s)}
        <label>
          <input type="radio" bind:group={tweaks.sidebar} value={s} />
          {s}
        </label>
      {/each}
    </fieldset>

    <fieldset>
      <legend>Scenario</legend>
      {#each scenarios as s (s)}
        <label>
          <input type="radio" bind:group={tweaks.scenario} value={s} />
          {s}
        </label>
      {/each}
    </fieldset>

    <fieldset>
      <legend>Page font</legend>
      <select bind:value={tweaks.pageFont}>
        {#each fonts as f (f)}
          <option value={f}>{f}</option>
        {/each}
      </select>
    </fieldset>

    <fieldset>
      <legend>Embeddings</legend>
      <div class="row">
        <span class="status">
          {#if embedderState.status?.local}
            <span class="dot ready"></span>
            BGE-small-en-v1.5
          {:else}
            <span class="dot fallback"></span>
            hash-bag (fallback)
          {/if}
        </span>
        {#if !embedderState.status?.local}
          <button
            class="primary"
            onclick={downloadModel}
            disabled={embedderState.busy}
            title="~130 MB; downloads to your app data directory"
          >
            {embedderState.busy ? "downloading…" : "download BGE-small"}
          </button>
        {/if}
      </div>
      {#if embedderState.error}
        <p class="err">{embedderState.error}</p>
      {/if}
    </fieldset>
  {/if}
</div>

<style>
  .dev-panel {
    position: fixed;
    bottom: 16px;
    right: 16px;
    z-index: 1000;
    background: rgba(20, 20, 22, 0.92);
    color: #ddd;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 12px 14px;
    font-family: "Inter", system-ui, sans-serif;
    font-size: 12px;
    width: 220px;
    backdrop-filter: blur(10px);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.4);
  }
  .dev-panel.closed {
    width: auto;
    padding: 0;
    background: transparent;
    border: 0;
    box-shadow: none;
  }
  .toggle {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 22px;
    height: 22px;
    border: 0;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
    color: #ddd;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .dev-panel.closed .toggle {
    position: static;
    background: rgba(20, 20, 22, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.08);
    width: 36px;
    height: 36px;
    border-radius: 50%;
    font-size: 16px;
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.4);
  }
  .toggle:hover {
    background: rgba(255, 255, 255, 0.12);
  }
  h4 {
    margin: 0 0 4px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: #aaa;
  }
  .hint {
    margin: 0 0 10px;
    font-size: 10.5px;
    color: #888;
    line-height: 1.4;
  }
  fieldset {
    border: 0;
    padding: 0;
    margin: 0 0 10px;
  }
  legend {
    font-size: 10.5px;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 4px;
    padding: 0;
  }
  label {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-right: 8px;
    font-size: 11.5px;
    color: #ccc;
    cursor: pointer;
  }
  input[type="radio"] {
    margin: 0;
    accent-color: #d6a464;
  }
  select {
    width: 100%;
    padding: 4px 6px;
    background: rgba(255, 255, 255, 0.05);
    color: #ddd;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    font-size: 11.5px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    margin-top: 2px;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: #ccc;
    font-size: 11px;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #888;
  }
  .dot.ready {
    background: #d6a464;
  }
  .dot.fallback {
    background: #888;
  }
  button.primary {
    background: rgba(214, 164, 100, 0.18);
    color: #d6a464;
    border: 1px solid rgba(214, 164, 100, 0.4);
    border-radius: 5px;
    font: inherit;
    font-size: 10.5px;
    padding: 3px 8px;
    cursor: pointer;
  }
  button.primary:hover {
    background: rgba(214, 164, 100, 0.28);
  }
  button.primary:disabled {
    opacity: 0.6;
    cursor: progress;
  }
  .err {
    margin-top: 6px;
    font-size: 10.5px;
    color: #c46c4c;
  }
</style>
