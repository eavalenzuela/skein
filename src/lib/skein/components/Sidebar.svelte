<script lang="ts">
  import { onMount } from "svelte";
  import type { SidebarMode } from "../tweaks.svelte.js";
  import { chatState, send, attachChatBus, CHAT_MODELS, type ContextMode } from "../chat.svelte.js";
  import { activeTab } from "../tabs.svelte.js";
  import { hasSecret } from "../settings.js";
  import { openSettings } from "../settingsUi.svelte.js";
  import ChatMessages from "./ChatMessages.svelte";

  interface Props {
    mode: SidebarMode;
  }
  let { mode }: Props = $props();

  let input = $state("");
  let keyConfigured = $state<boolean | null>(null);

  onMount(async () => {
    await attachChatBus();
    keyConfigured = await hasSecret("anthropic_api_key").catch(() => false);
  });

  async function refreshKey() {
    keyConfigured = await hasSecret("anthropic_api_key").catch(() => false);
  }

  async function onSubmit() {
    if (!input.trim() || chatState.busy) return;
    if (!keyConfigured) {
      await refreshKey();
      if (!keyConfigured) {
        openSettings();
        return;
      }
    }
    const text = input;
    input = "";
    const a = activeTab();
    await send(text, a?.rel_path ?? null);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void onSubmit();
    }
  }

  function modelLabel(id: string): string {
    return CHAT_MODELS.find((m) => m.id === id)?.label ?? id;
  }

  function cycleModel() {
    const idx = CHAT_MODELS.findIndex((m) => m.id === chatState.model);
    chatState.model = CHAT_MODELS[(idx + 1) % CHAT_MODELS.length].id;
  }

  const CONTEXT_OPTIONS: { id: ContextMode; label: string }[] = [
    { id: "current", label: "Current" },
    { id: "current+related", label: "Current + related" },
    { id: "vault", label: "Whole vault" },
  ];

  function cycleContext() {
    const idx = CONTEXT_OPTIONS.findIndex((c) => c.id === chatState.contextMode);
    chatState.contextMode = CONTEXT_OPTIONS[(idx + 1) % CONTEXT_OPTIONS.length].id;
  }

  function contextLabel(): string {
    return (
      CONTEXT_OPTIONS.find((c) => c.id === chatState.contextMode)?.label ?? chatState.contextMode
    );
  }
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
    <div class="col-pill">{modelLabel(chatState.model)} · {contextLabel()}</div>
  </div>
{:else if mode === "open"}
  <div class="sk-side">
    <div class="sk-side-hd">
      <button class="sk-pill bare" onclick={cycleModel} title="Click to change model">
        <span class="dot"></span>
        {modelLabel(chatState.model)}
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
      </button>
      <button
        class="sk-pill context bare"
        onclick={cycleContext}
        title="Click to change context mode"
      >
        {contextLabel()}
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
      </button>
      <div class="sk-side-spacer"></div>
    </div>

    {#if keyConfigured === false}
      <div class="key-hint">
        <p>Add your Anthropic API key in Settings to use the chat.</p>
        <button onclick={openSettings}>Open Settings</button>
      </div>
    {:else}
      <ChatMessages />
    {/if}

    <div class="sk-input">
      <textarea
        bind:value={input}
        placeholder={chatState.messages.length === 0
          ? "Ask Claude about this note"
          : "Continue the conversation"}
        onkeydown={onKey}
        rows="2"
        disabled={chatState.busy}
      ></textarea>
      <div class="sk-input-btns">
        <div class="left">
          <button disabled>@ Mention</button>
        </div>
        <button class="send" onclick={onSubmit} disabled={chatState.busy || !input.trim()}>
          <svg width="11" height="11" viewBox="0 0 14 14" fill="currentColor">
            <path d="M2 2l10 5-10 5 2-5z" />
          </svg>
          {chatState.busy ? "…" : "Send"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .sk-side textarea {
    width: 100%;
    background: transparent;
    border: 0;
    color: var(--ink);
    font-family: "Inter", system-ui, sans-serif;
    font-size: 12.5px;
    line-height: 1.5;
    resize: none;
    outline: none;
    padding: 0;
    min-height: 38px;
  }
  .sk-side textarea::placeholder {
    color: var(--ink-3);
  }
  .sk-side textarea:disabled {
    color: var(--ink-3);
    cursor: progress;
  }
  .sk-pill.bare {
    background: oklch(from var(--chrome-2) calc(l + 0.02) c h);
    cursor: pointer;
  }
  .sk-pill.bare:hover {
    background: oklch(from var(--chrome-2) calc(l + 0.05) c h);
  }
  .key-hint {
    flex: 1;
    overflow: auto;
    padding: 16px 14px;
    color: var(--ink-3);
    font-size: 12px;
    line-height: 1.5;
  }
  .key-hint p {
    margin: 0 0 10px;
  }
  .key-hint button {
    padding: 6px 10px;
    background: var(--accent-soft);
    color: var(--accent);
    border: 1px solid var(--accent-edge);
    border-radius: 5px;
    font: inherit;
    font-size: 11.5px;
    cursor: pointer;
  }
  .key-hint button:hover {
    background: oklch(from var(--accent) l c h / 0.28);
  }
</style>
