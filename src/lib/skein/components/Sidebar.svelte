<script lang="ts">
  import type { SidebarMode } from "../tweaks.svelte.js";
  import ChatMessages from "./ChatMessages.svelte";

  interface Props {
    mode: SidebarMode;
    withSelection?: boolean;
    streaming?: boolean;
  }
  let { mode, withSelection = false, streaming = false }: Props = $props();
</script>

{#if mode === "collapsed"}
  <div class="sk-side collapsed">
    <div class="col-icon">
      <svg
        width="13"
        height="13"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.3"
      >
        <rect x="2" y="3" width="12" height="10" rx="1.2" />
        <path d="M10 3v10" />
      </svg>
    </div>
    <div class="col-icon">
      <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 1l1.4 4.3 4.6.1-3.7 2.7L11.7 13 8 10.3 4.3 13 5.7 8.1 2 5.4l4.6-.1z" />
      </svg>
    </div>
    <div class="col-pill">Haiku 4.5 · Current + related</div>
  </div>
{:else if mode === "open"}
  <div class="sk-side">
    <div class="sk-side-hd">
      <div class="sk-pill">
        <span class="dot"></span> Haiku 4.5
        <span class="chev">
          <svg
            width="9"
            height="9"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
          >
            <path d="M3 5l3 3 3-3" />
          </svg>
        </span>
      </div>
      <div class="sk-pill context">
        Current + related
        <span class="chev">
          <svg
            width="9"
            height="9"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
          >
            <path d="M3 5l3 3 3-3" />
          </svg>
        </span>
      </div>
      <div class="sk-side-spacer"></div>
      <div class="sk-side-icon" title="History">
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
      <div class="sk-side-icon" title="Collapse">
        <svg
          width="13"
          height="13"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
        >
          <rect x="2" y="3" width="12" height="10" rx="1.2" />
          <path d="M10 3v10" />
        </svg>
      </div>
    </div>
    <ChatMessages {withSelection} {streaming} />
    <div class="sk-input">
      <div class="ph">Ask Claude about this note<span class="cursor"></span></div>
      <div class="sk-input-btns">
        <div class="left">
          <button>
            <svg
              width="13"
              height="13"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              stroke-width="1.4"
            >
              <path d="M11 5l-5.5 5.5a2.5 2.5 0 003.5 3.5L14 8.5a4 4 0 10-5.7-5.7L3 8" />
            </svg>
            Attach
          </button>
          <button>@ Mention</button>
        </div>
        <button class="send">
          <svg width="11" height="11" viewBox="0 0 14 14" fill="currentColor">
            <path d="M2 2l10 5-10 5 2-5z" />
          </svg>
          Send
        </button>
      </div>
    </div>
  </div>
{/if}
