<script lang="ts">
  import { onMount } from "svelte";
  import SkeinWindow from "$lib/skein/SkeinWindow.svelte";
  import VaultWindow from "$lib/skein/VaultWindow.svelte";
  import VaultPicker from "$lib/skein/components/VaultPicker.svelte";
  import DevControls from "$lib/skein/DevControls.svelte";
  import { tweaks, bootstrap as bootstrapTweaks, persist } from "$lib/skein/tweaks.svelte.js";
  import { vaultState, bootstrap } from "$lib/skein/vault.svelte.js";
  import { bootstrap as bootstrapEmbedder } from "$lib/skein/embedder.svelte.js";

  let bootstrapped = $state(false);
  let designPreview = $state(false);

  onMount(async () => {
    await Promise.all([bootstrapTweaks(), bootstrap(), bootstrapEmbedder()]);
    bootstrapped = true;
  });

  // Persist any change to the appearance tweaks (regardless of which UI
  // surface — Settings modal or DevControls — drives it). Scenario is
  // intentionally excluded; it's a dev-preview toggle and shouldn't be
  // saved across runs.
  $effect(() => {
    void [tweaks.theme, tweaks.shelfStyle, tweaks.sidebar, tweaks.pageFont];
    if (bootstrapped) persist();
  });
</script>

<div class="app-frame">
  {#if !bootstrapped}
    <div class="loading">…</div>
  {:else if vaultState.vault}
    <VaultWindow
      theme={tweaks.theme}
      shelfStyle={tweaks.shelfStyle}
      sidebar={tweaks.sidebar}
      pageFont={tweaks.pageFont}
    />
  {:else if designPreview}
    <SkeinWindow
      theme={tweaks.theme}
      shelfStyle={tweaks.shelfStyle}
      sidebar={tweaks.sidebar}
      scenario={tweaks.scenario}
      pageFont={tweaks.pageFont}
      vault="Field Notes"
    />
  {:else}
    <VaultPicker />
    <button class="preview-link" onclick={() => (designPreview = true)}
      >Skip — see the design preview</button
    >
  {/if}
</div>

<DevControls />

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    background: #1a1a1c;
    overflow: hidden;
  }
  :global(*) {
    box-sizing: border-box;
  }
  .app-frame {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }
  .loading {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    color: oklch(0.6 0.012 70);
    background: oklch(0.18 0.006 60);
    font-family: "Inter", system-ui, sans-serif;
    font-size: 18px;
  }
  .preview-link {
    position: fixed;
    bottom: 16px;
    left: 16px;
    background: rgba(255, 255, 255, 0.06);
    color: oklch(0.6 0.012 70);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 6px 12px;
    font-family: "Inter", system-ui, sans-serif;
    font-size: 11.5px;
    cursor: pointer;
    z-index: 1001;
  }
  .preview-link:hover {
    color: oklch(0.92 0.012 80);
    background: rgba(255, 255, 255, 0.1);
  }
</style>
