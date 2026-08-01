<script lang="ts">
  import { onMount } from "svelte";
  import type { SidebarMode } from "../tweaks.svelte.js";
  import {
    chatState,
    send,
    cancelActive,
    attachChatBus,
    CHAT_MODELS,
    type ContextMode,
  } from "../chat.svelte.js";
  import { activeTab } from "../tabs.svelte.js";
  import { hasSecret, getSettings, setSettings } from "../settings.js";
  import { openSettings } from "../settingsUi.svelte.js";
  import { titlesState } from "../titles.svelte.js";
  import ChatMessages from "./ChatMessages.svelte";
  import ContextMenu from "./ContextMenu.svelte";

  interface Props {
    mode: SidebarMode;
  }
  let { mode }: Props = $props();

  let input = $state("");
  let keyConfigured = $state<boolean | null>(null);
  let prefsError = $state<string | null>(null);
  let prefsSaved = $state(false);
  let prefsSavedTimer: ReturnType<typeof setTimeout> | null = null;
  let textareaEl: HTMLTextAreaElement | undefined = $state();
  let mentionOpen = $state(false);
  let mentionQuery = $state("");
  let mentionActive = $state(0);
  let mentionAnchor = $state(0); // position of the leading "@"

  let mentionResults = $derived(
    mentionOpen
      ? filterTitles(mentionQuery).slice(0, 8)
      : ([] as { rel_path: string; title: string }[]),
  );

  function filterTitles(q: string) {
    const items = titlesState.items;
    if (!q) return items.slice(0, 8);
    const term = q.toLowerCase();
    return items.filter((t) => t.title.toLowerCase().includes(term));
  }

  function openMentionAtCursor() {
    if (!textareaEl) return;
    const pos = textareaEl.selectionStart ?? input.length;
    // Insert "@" at cursor and arm the picker.
    input = input.slice(0, pos) + "@" + input.slice(pos);
    mentionAnchor = pos;
    mentionQuery = "";
    mentionOpen = true;
    mentionActive = 0;
    requestAnimationFrame(() => {
      textareaEl?.focus();
      textareaEl?.setSelectionRange(pos + 1, pos + 1);
    });
  }

  function syncMentionFromInput() {
    if (!mentionOpen || !textareaEl) return;
    const pos = textareaEl.selectionStart ?? input.length;
    if (pos < mentionAnchor + 1 || input[mentionAnchor] !== "@") {
      mentionOpen = false;
      return;
    }
    const between = input.slice(mentionAnchor + 1, pos);
    if (/\s/.test(between)) {
      mentionOpen = false;
      return;
    }
    mentionQuery = between;
    mentionActive = 0;
  }

  function chooseMention(t: { title: string }) {
    if (!textareaEl || !mentionOpen) return;
    const before = input.slice(0, mentionAnchor);
    const cursorPos = textareaEl.selectionStart ?? input.length;
    const after = input.slice(cursorPos);
    const insertion = `[[${t.title}]]`;
    input = before + insertion + after;
    mentionOpen = false;
    const newPos = mentionAnchor + insertion.length;
    requestAnimationFrame(() => {
      textareaEl?.focus();
      textareaEl?.setSelectionRange(newPos, newPos);
    });
  }

  onMount(async () => {
    await attachChatBus();
    keyConfigured = await hasSecret("anthropic_api_key").catch(() => false);
    // Restore last-used model + context mode so users don't have to re-pick
    // every launch. Saved on each cycle below.
    try {
      const s = await getSettings();
      if (s.chat_model && CHAT_MODELS.some((m) => m.id === s.chat_model)) {
        chatState.model = s.chat_model;
      }
      if (
        s.chat_context_mode &&
        (["current", "current+related", "vault"] as const).includes(
          s.chat_context_mode as ContextMode,
        )
      ) {
        chatState.contextMode = s.chat_context_mode as ContextMode;
      }
    } catch {
      // Settings load failed — fall back to defaults. Non-fatal.
    }
  });

  async function persistChatPrefs() {
    prefsError = null;
    try {
      await setSettings({
        chat_model: chatState.model,
        chat_context_mode: chatState.contextMode,
      });
      prefsSaved = true;
      if (prefsSavedTimer) clearTimeout(prefsSavedTimer);
      prefsSavedTimer = setTimeout(() => (prefsSaved = false), 1500);
    } catch (e) {
      // Surface so the user knows their model/context choice didn't stick.
      prefsError = `Couldn't save preference: ${String(e)}`;
    }
  }

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
    if (mentionOpen) {
      if (e.key === "Escape") {
        e.preventDefault();
        mentionOpen = false;
        return;
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        if (mentionResults.length > 0)
          mentionActive = (mentionActive + 1) % mentionResults.length;
        return;
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        if (mentionResults.length > 0)
          mentionActive = (mentionActive - 1 + mentionResults.length) % mentionResults.length;
        return;
      } else if (e.key === "Enter" || e.key === "Tab") {
        const pick = mentionResults[mentionActive];
        if (pick) {
          e.preventDefault();
          chooseMention(pick);
          return;
        }
      }
    }
    if (e.key === "@") {
      // Defer: textarea will insert the @, then we arm the picker.
      requestAnimationFrame(() => {
        if (!textareaEl) return;
        mentionAnchor = (textareaEl.selectionStart ?? 1) - 1;
        if (input[mentionAnchor] !== "@") return;
        mentionQuery = "";
        mentionOpen = true;
        mentionActive = 0;
      });
      return;
    }
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void onSubmit();
    }
  }

  function onInput() {
    syncMentionFromInput();
  }

  function modelLabel(id: string): string {
    return CHAT_MODELS.find((m) => m.id === id)?.label ?? id;
  }

  // Both pills carry a chevron, so they must actually open a menu. They
  // used to advance to the next value on click — a user clicking to see
  // the options instead silently changed them, possibly onto Opus.
  let modelMenu = $state<{ x: number; y: number } | null>(null);
  let contextMenu = $state<{ x: number; y: number } | null>(null);

  function anchorOf(e: MouseEvent): { x: number; y: number } {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    return { x: rect.left, y: rect.bottom + 3 };
  }

  const CONTEXT_OPTIONS: { id: ContextMode; label: string; hint: string }[] = [
    { id: "current", label: "Current", hint: "this page only" },
    { id: "current+related", label: "Current + related", hint: "plus similar notes" },
    { id: "vault", label: "Whole vault", hint: "widest search, most tokens" },
  ];

  let modelItems = $derived(
    CHAT_MODELS.map((m) => ({
      label: m.label,
      hint: m.id === chatState.model ? "current" : undefined,
      action: () => {
        chatState.model = m.id;
        void persistChatPrefs();
      },
    })),
  );

  let contextItems = $derived(
    CONTEXT_OPTIONS.map((c) => ({
      label: c.label,
      hint: c.id === chatState.contextMode ? "current" : c.hint,
      action: () => {
        chatState.contextMode = c.id;
        void persistChatPrefs();
      },
    })),
  );

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
      <button
        class="sk-pill bare"
        onclick={(e) => (modelMenu = anchorOf(e))}
        aria-haspopup="menu"
        title="Choose the chat model"
      >
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
        onclick={(e) => (contextMenu = anchorOf(e))}
        aria-haspopup="menu"
        title="Choose how much of the vault to send"
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
      {#if prefsSaved}
        <span class="prefs-flash" aria-live="polite">saved</span>
      {/if}
      {#if prefsError}
        <span class="prefs-error" title={prefsError} aria-live="polite">! save failed</span>
      {/if}
    </div>

    {#if modelMenu}
      <ContextMenu
        x={modelMenu.x}
        y={modelMenu.y}
        items={modelItems}
        onclose={() => (modelMenu = null)}
      />
    {/if}
    {#if contextMenu}
      <ContextMenu
        x={contextMenu.x}
        y={contextMenu.y}
        items={contextItems}
        onclose={() => (contextMenu = null)}
      />
    {/if}

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
        bind:this={textareaEl}
        bind:value={input}
        placeholder={chatState.messages.length === 0
          ? "Ask Claude about this note"
          : "Continue the conversation"}
        onkeydown={onKey}
        oninput={onInput}
        rows="2"
        disabled={chatState.busy}
      ></textarea>
      {#if mentionOpen && mentionResults.length > 0}
        <ul class="mention-popup" role="listbox" aria-label="Mention a page">
          {#each mentionResults as item, i (item.rel_path)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <li
              class:active={i === mentionActive}
              onclick={() => chooseMention(item)}
              onmouseenter={() => (mentionActive = i)}
              role="option"
              aria-selected={i === mentionActive}
            >
              <span class="mt">{item.title}</span>
              <span class="mr">{item.rel_path}</span>
            </li>
          {/each}
        </ul>
      {/if}
      <div class="sk-input-btns">
        <div class="left">
          <button
            onclick={openMentionAtCursor}
            title="Mention a page (or type @ in the message)"
            disabled={chatState.busy}>@ Mention</button
          >
        </div>
        {#if chatState.busy}
          <button
            class="send stop"
            onclick={() => void cancelActive()}
            title="Stop generating"
          >
            <svg width="11" height="11" viewBox="0 0 14 14" fill="currentColor">
              <rect x="3" y="3" width="8" height="8" rx="1" />
            </svg>
            Stop
          </button>
        {:else}
          <button class="send" onclick={onSubmit} disabled={!input.trim()}>
            <svg width="11" height="11" viewBox="0 0 14 14" fill="currentColor">
              <path d="M2 2l10 5-10 5 2-5z" />
            </svg>
            Send
          </button>
        {/if}
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
    max-height: 240px;
    overflow-y: auto;
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
  .send.stop {
    background: oklch(0.45 0.16 25);
    border-color: oklch(0.55 0.16 25);
    color: oklch(0.96 0.02 60);
  }
  .send.stop:hover {
    filter: brightness(1.12);
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
  .prefs-flash {
    font-size: 10.5px;
    color: var(--accent);
    font-style: italic;
    align-self: center;
    animation: fade-in 120ms ease-in;
  }
  .prefs-error {
    font-size: 10.5px;
    color: oklch(0.65 0.18 25);
    align-self: center;
    cursor: help;
  }
  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  .mention-popup {
    list-style: none;
    margin: 0;
    padding: 4px 0;
    background: var(--chrome-2);
    border: 1px solid var(--chrome-edge);
    border-radius: 6px;
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.45);
    max-height: 220px;
    overflow: auto;
    margin-top: 6px;
  }
  .mention-popup li {
    padding: 6px 10px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .mention-popup li.active {
    background: var(--accent-soft);
  }
  .mention-popup .mt {
    font-size: 12px;
    color: var(--ink);
  }
  .mention-popup .mr {
    font-family: "JetBrains Mono", monospace;
    font-size: 10px;
    color: var(--ink-3);
  }
</style>
