<script lang="ts" module>
  // A themed replacement for window.prompt, which is unstyled, blocks the
  // renderer, and is a no-op in macOS WKWebView. Any surface that needs a
  // page title opens this instead.
  //
  // Resolves with the trimmed title, or null if the user backs out.
  let resolver: ((v: string | null) => void) | null = null;

  export const newPagePrompt: { open: boolean; book: string | null } = $state({
    open: false,
    book: null,
  });

  export function askForPageTitle(book: string | null): Promise<string | null> {
    resolver?.(null);
    newPagePrompt.book = book;
    newPagePrompt.open = true;
    return new Promise((resolve) => {
      resolver = resolve;
    });
  }

  export function settlePagePrompt(value: string | null) {
    newPagePrompt.open = false;
    const r = resolver;
    resolver = null;
    r?.(value);
  }
</script>

<script lang="ts">
  import { focusTrap } from "../focusTrap.js";

  let value = $state("");

  function submit() {
    const t = value.trim();
    if (!t) return;
    value = "";
    settlePagePrompt(t);
  }

  function cancel() {
    value = "";
    settlePagePrompt(null);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={cancel}>
  <div
    class="sheet"
    role="dialog"
    aria-modal="true"
    aria-label="New page"
    tabindex="-1"
    use:focusTrap
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => {
      if (e.key === "Escape") cancel();
    }}
  >
    <h2>New page{newPagePrompt.book ? ` in ${newPagePrompt.book}` : " in Folio"}</h2>
    <form
      onsubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <!-- svelte-ignore a11y_autofocus -->
      <input bind:value placeholder="Page title" aria-label="Page title" autofocus />
      <div class="actions">
        <button type="button" onclick={cancel}>Cancel</button>
        <button type="submit" class="primary" disabled={!value.trim()}>Create</button>
      </div>
    </form>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: oklch(0 0 0 / 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 24px;
  }
  .sheet {
    width: min(420px, 100%);
    background: var(--chrome);
    border: 1px solid var(--chrome-edge);
    border-radius: 10px;
    padding: 18px;
    box-shadow: 0 24px 60px -20px oklch(0 0 0 / 0.7);
    outline: none;
  }
  h2 {
    margin: 0 0 12px;
    font-family: var(--page-font, "Source Serif 4"), serif;
    font-size: 16px;
    font-weight: 500;
    color: var(--ink);
  }
  input {
    width: 100%;
    background: oklch(from var(--chrome-2) calc(l + 0.03) c h);
    border: 1px solid var(--chrome-edge);
    color: var(--ink);
    border-radius: 6px;
    padding: 7px 10px;
    font-family: "Inter", sans-serif;
    font-size: 13px;
    outline: none;
  }
  input:focus {
    border-color: var(--accent-edge);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 12px;
  }
  button {
    background: var(--chrome-2);
    border: 1px solid var(--chrome-edge);
    color: var(--ink-2);
    border-radius: 6px;
    padding: 6px 14px;
    font-family: "Inter", sans-serif;
    font-size: 12px;
    cursor: pointer;
  }
  button:hover:not(:disabled) {
    color: var(--ink);
  }
  button.primary {
    background: var(--accent-soft);
    border-color: var(--accent-edge);
    color: var(--ink);
  }
  button.primary:hover:not(:disabled) {
    background: var(--accent);
    color: var(--chrome);
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
