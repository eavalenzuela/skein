<script lang="ts">
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { open as openVaultPath, vaultState } from "../vault.svelte.js";

  async function pick() {
    const selected = await openDialog({ directory: true, multiple: false });
    if (typeof selected === "string") {
      await openVaultPath(selected);
    }
  }
</script>

<div class="vault-picker">
  <div class="card">
    <h1>Skein</h1>
    <p>Pick a folder to use as your vault.</p>
    <p class="hint">
      Books are subfolders, pages are <code>.md</code> files. Top-level pages live in the Folio.
    </p>
    <button onclick={pick} disabled={vaultState.loading}>
      {vaultState.loading ? "Opening…" : "Choose vault folder"}
    </button>
    {#if vaultState.error}
      <p class="error">{vaultState.error}</p>
    {/if}
  </div>
</div>

<style>
  .vault-picker {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100vw;
    height: 100vh;
    background: oklch(0.18 0.006 60);
    color: oklch(0.92 0.012 80);
    font-family: "Inter", system-ui, sans-serif;
  }
  .card {
    background: oklch(0.22 0.008 60);
    border: 1px solid oklch(0.13 0.005 55);
    border-radius: 10px;
    padding: 36px 40px;
    width: 480px;
    text-align: center;
    box-shadow: 0 30px 80px -20px oklch(0 0 0 / 0.55);
  }
  h1 {
    font-family: "Source Serif 4", Georgia, serif;
    font-weight: 600;
    margin: 0 0 18px;
    font-size: 32px;
    letter-spacing: -0.01em;
  }
  p {
    margin: 0 0 14px;
    font-size: 13px;
    color: oklch(0.78 0.015 75);
    line-height: 1.5;
  }
  .hint {
    font-size: 11.5px;
    color: oklch(0.6 0.012 70);
    margin-bottom: 22px;
  }
  code {
    font-family: "JetBrains Mono", monospace;
    font-size: 11.5px;
    background: oklch(0.27 0.011 70);
    padding: 1px 5px;
    border-radius: 3px;
  }
  button {
    background: oklch(0.78 0.13 75);
    color: oklch(0.2 0.02 60);
    border: 0;
    border-radius: 6px;
    padding: 9px 18px;
    font: inherit;
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
  }
  button:hover {
    filter: brightness(1.05);
  }
  button:disabled {
    opacity: 0.6;
    cursor: progress;
  }
  .error {
    margin-top: 16px;
    color: oklch(0.65 0.18 25);
    font-size: 12px;
  }
</style>
