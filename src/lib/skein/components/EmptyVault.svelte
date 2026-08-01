<script lang="ts">
  // The real app's empty desk. EmptyDesk renders fixture cards from
  // data.ts, which belong to the design mockup (Desk.svelte) — showing
  // them here presents five notes the user never wrote, none of which do
  // anything when clicked. This is the same moment with real affordances.
  import { openSearch } from "../searchUi.svelte.js";
  import { openTodayDaily } from "../vault.js";
  import { openTab } from "../tabs.svelte.js";
  import { vaultState, refreshVault } from "../vault.svelte.js";
  import { createPage } from "../vault.js";
  import { toastError } from "../toasts.svelte.js";

  let creating = $state(false);
  let newTitle = $state("");

  async function create() {
    const title = newTitle.trim();
    if (!title) return;
    try {
      const rel = await createPage(null, title);
      await refreshVault();
      await openTab({ rel_path: rel, title });
      newTitle = "";
      creating = false;
    } catch (e) {
      toastError("Couldn't create the page", String(e));
    }
  }

  async function daily() {
    try {
      const res = await openTodayDaily();
      const stem = res.rel_path.split("/").pop()?.replace(/\.md$/, "") ?? res.rel_path;
      await refreshVault();
      await openTab({ rel_path: res.rel_path, title: stem });
    } catch (e) {
      toastError("Couldn't open today's daily note", String(e));
    }
  }

  let bookCount = $derived(vaultState.books.length);
</script>

<div class="empty">
  <div class="inner">
    <h2>The desk is clear.</h2>
    <p class="sub">
      {#if bookCount > 0}
        Pull a book off the shelf above, or start something new.
      {:else}
        Nothing in this vault yet. Books are folders; pages are markdown files.
      {/if}
    </p>

    {#if creating}
      <form
        class="create"
        onsubmit={(e) => {
          e.preventDefault();
          void create();
        }}
      >
        <!-- svelte-ignore a11y_autofocus -->
        <input
          bind:value={newTitle}
          placeholder="Page title"
          aria-label="New page title"
          autofocus
          onkeydown={(e) => {
            if (e.key === "Escape") creating = false;
          }}
        />
        <button type="submit" class="primary" disabled={!newTitle.trim()}>Create</button>
        <button type="button" onclick={() => (creating = false)}>Cancel</button>
      </form>
    {:else}
      <div class="actions">
        <button class="primary" onclick={() => (creating = true)}>New page</button>
        <button onclick={() => void daily()}>Today's daily note</button>
        <button onclick={openSearch}>Search<kbd>Ctrl</kbd><kbd>K</kbd></button>
      </div>
    {/if}
  </div>
</div>

<style>
  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 32px;
  }
  .inner {
    text-align: center;
    max-width: 46ch;
  }
  h2 {
    margin: 0 0 6px;
    font-family: var(--page-font, "Source Serif 4"), serif;
    font-size: 21px;
    font-weight: 500;
    color: var(--ink-2);
  }
  .sub {
    margin: 0 0 18px;
    font-family: "Inter", sans-serif;
    font-size: 12.5px;
    color: var(--ink-3);
    line-height: 1.5;
  }
  .actions,
  .create {
    display: flex;
    gap: 8px;
    justify-content: center;
    flex-wrap: wrap;
  }
  button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: var(--chrome-2);
    border: 1px solid var(--chrome-edge);
    color: var(--ink-2);
    border-radius: 6px;
    padding: 6px 13px;
    font-family: "Inter", sans-serif;
    font-size: 12px;
    cursor: pointer;
  }
  button:hover {
    color: var(--ink);
    border-color: var(--accent-edge);
  }
  button.primary {
    background: var(--accent-soft);
    border-color: var(--accent-edge);
    color: var(--ink);
  }
  button.primary:hover {
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
  kbd {
    font-family: "JetBrains Mono", monospace;
    font-size: 9.5px;
    background: oklch(1 0 0 / 0.07);
    border: 1px solid var(--chrome-edge);
    border-radius: 3px;
    padding: 1px 4px;
    color: var(--ink-3);
  }
  input {
    background: var(--page);
    border: 1px solid var(--page-edge);
    color: var(--ink);
    border-radius: 6px;
    padding: 6px 10px;
    font-family: "Inter", sans-serif;
    font-size: 12px;
    min-width: 200px;
  }
</style>
