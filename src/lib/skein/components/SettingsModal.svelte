<script lang="ts">
  import { onMount } from "svelte";
  import {
    tweaks,
    persist,
    type Theme,
    type ShelfStyle,
    type SidebarMode,
    type PageFont,
  } from "../tweaks.svelte.js";
  import { embedderState, downloadModel } from "../embedder.svelte.js";
  import { vaultState, open as openVaultPath, close as closeVault } from "../vault.svelte.js";
  import { hasSecret, setSecret, clearSecret, type SecretName } from "../settings.js";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  const themes: Theme[] = ["dark", "light"];
  const shelves: ShelfStyle[] = ["abstract", "suggestive", "tactile"];
  const sidebars: SidebarMode[] = ["open", "collapsed", "hidden"];
  const fonts: PageFont[] = ["Source Serif 4", "Iowan Old Style", "Spectral", "Lora"];

  let anthropicSet = $state(false);
  let voyageSet = $state(false);
  let anthropicInput = $state("");
  let voyageInput = $state("");
  let savingAnthropic = $state(false);
  let savingVoyage = $state(false);
  let secretError = $state<string | null>(null);

  async function refreshSecrets() {
    try {
      [anthropicSet, voyageSet] = await Promise.all([
        hasSecret("anthropic_api_key"),
        hasSecret("voyage_api_key"),
      ]);
    } catch (e) {
      secretError = String(e);
    }
  }

  onMount(refreshSecrets);

  async function saveSecret(name: SecretName, value: string) {
    secretError = null;
    if (name === "anthropic_api_key") savingAnthropic = true;
    else savingVoyage = true;
    try {
      await setSecret(name, value);
      if (name === "anthropic_api_key") {
        anthropicInput = "";
        anthropicSet = true;
      } else {
        voyageInput = "";
        voyageSet = true;
      }
    } catch (e) {
      secretError = String(e);
    } finally {
      savingAnthropic = false;
      savingVoyage = false;
    }
  }

  async function clearAndForget(name: SecretName) {
    secretError = null;
    try {
      await clearSecret(name);
      if (name === "anthropic_api_key") anthropicSet = false;
      else voyageSet = false;
    } catch (e) {
      secretError = String(e);
    }
  }

  async function pickNewVault() {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await openVaultPath(selected);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div
    class="modal"
    role="dialog"
    aria-label="Settings"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
  >
    <header>
      <h2>Settings</h2>
      <button class="close" onclick={onClose} aria-label="Close">×</button>
    </header>

    <div class="body">
      <section>
        <h3>Vault</h3>
        {#if vaultState.vault}
          <div class="row">
            <div class="kv">
              <div class="k">{vaultState.vault.name}</div>
              <div class="v mono">{vaultState.vault.root}</div>
            </div>
            <div class="actions">
              <button onclick={pickNewVault}>change…</button>
              <button class="danger" onclick={closeVault}>close</button>
            </div>
          </div>
        {:else}
          <div class="row">
            <p class="muted">No vault open.</p>
            <button class="primary" onclick={pickNewVault}>pick a folder</button>
          </div>
        {/if}
      </section>

      <section>
        <h3>Appearance</h3>
        <div class="grid">
          <div class="grid-label">Theme</div>
          <div class="radios">
            {#each themes as t (t)}
              <label class="opt">
                <input type="radio" bind:group={tweaks.theme} value={t} onchange={persist} />
                {t}
              </label>
            {/each}
          </div>

          <div class="grid-label">Shelf realism</div>
          <div class="radios">
            {#each shelves as s (s)}
              <label class="opt">
                <input type="radio" bind:group={tweaks.shelfStyle} value={s} onchange={persist} />
                {s}
              </label>
            {/each}
          </div>

          <div class="grid-label">Sidebar default</div>
          <div class="radios">
            {#each sidebars as s (s)}
              <label class="opt">
                <input type="radio" bind:group={tweaks.sidebar} value={s} onchange={persist} />
                {s}
              </label>
            {/each}
          </div>

          <div class="grid-label">Page serif</div>
          <select bind:value={tweaks.pageFont} onchange={persist}>
            {#each fonts as f (f)}
              <option value={f}>{f}</option>
            {/each}
          </select>
        </div>
      </section>

      <section>
        <h3>Embeddings</h3>
        <div class="row">
          <div class="kv">
            <div class="k">
              {embedderState.status?.local
                ? "BGE-small-en-v1.5 (local ONNX)"
                : "hash-bag (fallback)"}
            </div>
            <div class="v">
              {embedderState.status?.local
                ? "Semantic similarity is on. Pages share neighbours by meaning."
                : "Keyword-overlap fallback. Download the local model for semantic neighbours."}
            </div>
          </div>
          {#if !embedderState.status?.local}
            <button class="primary" onclick={downloadModel} disabled={embedderState.busy}>
              {embedderState.busy ? "downloading…" : "download BGE-small (~130 MB)"}
            </button>
          {/if}
        </div>
        {#if embedderState.error}
          <p class="error">{embedderState.error}</p>
        {/if}
      </section>

      <section>
        <h3>API keys</h3>
        <p class="muted">
          Stored in your OS keychain — libsecret on Linux, Credential Manager on Windows. Skein
          never writes them to disk in plaintext and never returns them to the UI after they're
          saved.
        </p>

        <div class="key">
          <div class="key-info">
            <span class="k">Anthropic</span>
            <span class="v">Unlocks the chat sidebar and auto-tagging.</span>
          </div>
          <div class="key-input">
            {#if anthropicSet && !anthropicInput}
              <input type="text" disabled value="•••• configured ••••" />
              <button class="danger" onclick={() => clearAndForget("anthropic_api_key")}
                >forget</button
              >
            {:else}
              <input
                type="password"
                placeholder={anthropicSet ? "enter a new key to replace" : "sk-ant-…"}
                bind:value={anthropicInput}
                autocomplete="off"
                spellcheck="false"
              />
              <button
                class="primary"
                onclick={() => saveSecret("anthropic_api_key", anthropicInput)}
                disabled={!anthropicInput || savingAnthropic}
              >
                {savingAnthropic ? "saving…" : "save"}
              </button>
            {/if}
          </div>
        </div>

        <div class="key">
          <div class="key-info">
            <span class="k">Voyage</span>
            <span class="v">Optional. Higher-quality remote embeddings (Phase 5c).</span>
          </div>
          <div class="key-input">
            {#if voyageSet && !voyageInput}
              <input type="text" disabled value="•••• configured ••••" />
              <button class="danger" onclick={() => clearAndForget("voyage_api_key")}>forget</button
              >
            {:else}
              <input
                type="password"
                placeholder={voyageSet ? "enter a new key to replace" : "pa-…"}
                bind:value={voyageInput}
                autocomplete="off"
                spellcheck="false"
              />
              <button
                class="primary"
                onclick={() => saveSecret("voyage_api_key", voyageInput)}
                disabled={!voyageInput || savingVoyage}
              >
                {savingVoyage ? "saving…" : "save"}
              </button>
            {/if}
          </div>
        </div>

        {#if secretError}
          <p class="error">{secretError}</p>
        {/if}
      </section>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: oklch(0 0 0 / 0.5);
    z-index: 800;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 6vh;
    backdrop-filter: blur(2px);
  }
  .modal {
    width: 720px;
    max-width: 92vw;
    max-height: 86vh;
    background: var(--chrome-2);
    border: 1px solid var(--chrome-edge);
    border-radius: 10px;
    color: var(--ink);
    font-family: "Inter", system-ui, sans-serif;
    box-shadow: 0 30px 80px -20px oklch(0 0 0 / 0.55);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid var(--chrome-edge);
    background: var(--chrome);
  }
  h2 {
    margin: 0;
    font-family: "Source Serif 4", Georgia, serif;
    font-weight: 600;
    font-size: 18px;
  }
  .close {
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: var(--ink-3);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }
  .close:hover {
    background: oklch(1 0 0 / 0.06);
    color: var(--ink);
  }
  .body {
    overflow: auto;
    padding: 12px 20px 20px;
  }
  section {
    padding: 16px 0;
    border-bottom: 1px solid var(--chrome-edge);
  }
  section:last-child {
    border-bottom: 0;
  }
  h3 {
    margin: 0 0 12px;
    font-size: 11.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--ink-3);
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .kv {
    flex: 1;
    min-width: 0;
  }
  .kv .k {
    font-size: 13px;
    color: var(--ink);
    margin-bottom: 2px;
  }
  .kv .v {
    font-size: 11.5px;
    color: var(--ink-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .kv .v.mono {
    font-family: "JetBrains Mono", monospace;
    font-size: 11px;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .grid {
    display: grid;
    grid-template-columns: 140px 1fr;
    column-gap: 16px;
    row-gap: 10px;
    align-items: center;
    font-size: 12px;
  }
  .grid > .grid-label {
    color: var(--ink-3);
    font-size: 11.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .radios {
    display: flex;
    gap: 14px;
    flex-wrap: wrap;
  }
  .opt {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    font-size: 12px;
    color: var(--ink-2);
  }
  input[type="radio"] {
    accent-color: var(--accent);
  }
  select {
    padding: 5px 8px;
    background: oklch(from var(--chrome-2) calc(l + 0.03) c h);
    color: var(--ink);
    border: 1px solid var(--chrome-edge);
    border-radius: 5px;
    font-size: 12px;
    width: max-content;
    min-width: 200px;
  }
  .key {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 14px;
    align-items: center;
    margin-bottom: 12px;
  }
  .key:last-of-type {
    margin-bottom: 0;
  }
  .key > .key-info {
    cursor: default;
  }
  .key .k {
    display: block;
    font-size: 13px;
    color: var(--ink);
    margin-bottom: 2px;
  }
  .key .v {
    display: block;
    font-size: 11px;
    color: var(--ink-3);
  }
  .key-input {
    display: flex;
    gap: 6px;
  }
  .key-input input {
    flex: 1;
    padding: 6px 10px;
    background: oklch(from var(--chrome-2) calc(l + 0.03) c h);
    border: 1px solid var(--chrome-edge);
    color: var(--ink);
    border-radius: 5px;
    font-family: "JetBrains Mono", monospace;
    font-size: 12px;
    outline: none;
  }
  .key-input input:focus {
    border-color: var(--accent-edge);
  }
  .key-input input:disabled {
    color: var(--ink-3);
    background: var(--chrome);
  }
  button {
    padding: 6px 10px;
    background: oklch(from var(--chrome-2) calc(l + 0.04) c h);
    color: var(--ink-2);
    border: 1px solid var(--chrome-edge);
    border-radius: 5px;
    font: inherit;
    font-size: 11.5px;
    cursor: pointer;
  }
  button:hover {
    color: var(--ink);
    background: oklch(from var(--chrome-2) calc(l + 0.07) c h);
  }
  button.primary {
    background: var(--accent-soft);
    color: var(--accent);
    border-color: var(--accent-edge);
  }
  button.primary:hover {
    background: oklch(from var(--accent) l c h / 0.28);
  }
  button.primary:disabled {
    opacity: 0.55;
    cursor: progress;
  }
  button.danger {
    color: oklch(0.7 0.15 25);
    border-color: oklch(0.7 0.15 25 / 0.4);
  }
  button.danger:hover {
    background: oklch(0.7 0.15 25 / 0.12);
  }
  .muted {
    color: var(--ink-3);
    font-size: 11.5px;
    line-height: 1.5;
    margin: 0 0 10px;
  }
  .error {
    margin-top: 10px;
    color: oklch(0.65 0.18 25);
    font-size: 11.5px;
  }
</style>
