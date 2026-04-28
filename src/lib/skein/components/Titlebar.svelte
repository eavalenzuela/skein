<script lang="ts">
  import { openSearch } from "../searchUi.svelte.js";
  import { openSettings } from "../settingsUi.svelte.js";

  interface Props {
    vault: string;
  }
  let { vault }: Props = $props();
</script>

<div class="sk-titlebar">
  <div class="sk-tb-menus">
    <div class="sk-tb-menu">File</div>
    <div class="sk-tb-menu">Edit</div>
    <div class="sk-tb-menu">View</div>
    <div class="sk-tb-menu">Help</div>
  </div>
  <div class="sk-tb-title">
    Skein <span class="vault">— {vault}</span>
  </div>
  <div class="sk-tb-right">
    <button class="sk-tb-btn bare" title="Search (Ctrl+K)" onclick={openSearch} aria-label="Search">
      <svg
        width="13"
        height="13"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.4"
      >
        <circle cx="7" cy="7" r="4" />
        <path d="M10 10l3 3" />
      </svg>
    </button>
    <div class="sk-tb-btn" title="Vaults">
      <svg
        width="14"
        height="14"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.3"
      >
        <path d="M3 2.5h10M3 5.5h10M3 8.5h7M3 11.5h5" />
      </svg>
    </div>
    <button
      class="sk-tb-btn bare"
      title="Settings (Ctrl+,)"
      onclick={openSettings}
      aria-label="Settings"
    >
      <svg
        width="13"
        height="13"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.4"
      >
        <circle cx="8" cy="8" r="2.2" />
        <path
          d="M8 1.5v2M8 12.5v2M14.5 8h-2M3.5 8h-2M12.6 3.4l-1.4 1.4M4.8 11.2l-1.4 1.4M12.6 12.6l-1.4-1.4M4.8 4.8L3.4 3.4"
        />
      </svg>
    </button>
    <div class="sk-tb-window">
      <button title="Minimize" aria-label="Minimize">
        <svg
          width="8"
          height="8"
          viewBox="0 0 8 8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
        >
          <path d="M1.5 4h5" />
        </svg>
      </button>
      <button title="Maximize" aria-label="Maximize">
        <svg
          width="8"
          height="8"
          viewBox="0 0 8 8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
        >
          <rect x="1.5" y="1.5" width="5" height="5" />
        </svg>
      </button>
      <button title="Close" class="close" aria-label="Close">
        <svg
          width="8"
          height="8"
          viewBox="0 0 8 8"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
        >
          <path d="M2 2l4 4M6 2l-4 4" />
        </svg>
      </button>
    </div>
  </div>
</div>

<style>
  .bare {
    background: transparent;
    border: 0;
    cursor: pointer;
    color: inherit;
    font: inherit;
  }
</style>
