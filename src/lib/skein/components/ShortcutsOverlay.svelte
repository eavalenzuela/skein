<script lang="ts">
  import { SHORTCUT_GROUPS } from "../shortcuts.js";
  import { focusTrap } from "../focusTrap.js";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  // Ctrl on Linux/Windows, ⌘ on macOS — the handlers accept either.
  const isMac =
    typeof navigator !== "undefined" && /mac/i.test(navigator.platform || navigator.userAgent);
  const key = (k: string) => (isMac && k === "Ctrl" ? "⌘" : k);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose}>
  <div
    class="sheet"
    role="dialog"
    aria-modal="true"
    aria-label="Keyboard shortcuts"
    tabindex="-1"
    use:focusTrap
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => {
      if (e.key !== "Escape") return;
      // Stop here: this overlay can sit on top of Settings, whose Escape
      // handler is bound to the window. Without this one press closed both.
      e.stopPropagation();
      onClose();
    }}
  >
    <header>
      <h2>Keyboard shortcuts</h2>
      <button class="x" onclick={onClose} aria-label="Close">×</button>
    </header>
    <div class="groups">
      {#each SHORTCUT_GROUPS as g (g.title)}
        <section>
          <h3>{g.title}</h3>
          <dl>
            {#each g.items as s, i (g.title + i)}
              <div class="row">
                <dt>
                  {#each s.keys as k, ki (ki)}
                    <kbd>{key(k)}</kbd>
                  {/each}
                </dt>
                <dd>{s.label}</dd>
              </div>
            {/each}
          </dl>
        </section>
      {/each}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: oklch(0 0 0 / 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 24px;
  }
  .sheet {
    width: min(760px, 100%);
    max-height: 82vh;
    display: flex;
    flex-direction: column;
    background: var(--chrome);
    border: 1px solid var(--chrome-edge);
    border-radius: 10px;
    box-shadow: 0 24px 60px -20px oklch(0 0 0 / 0.7);
    outline: none;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    border-bottom: 1px solid var(--chrome-edge);
  }
  h2 {
    margin: 0;
    font-family: var(--page-font, "Source Serif 4"), serif;
    font-size: 17px;
    font-weight: 500;
    color: var(--ink);
  }
  .x {
    background: transparent;
    border: 1px solid var(--chrome-edge);
    color: var(--ink-3);
    border-radius: 5px;
    width: 24px;
    height: 24px;
    font-size: 15px;
    line-height: 1;
    cursor: pointer;
  }
  .x:hover {
    color: var(--ink);
  }
  .groups {
    overflow-y: auto;
    padding: 16px 18px 20px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 18px 28px;
  }
  h3 {
    margin: 0 0 8px;
    font-family: "Inter", sans-serif;
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--ink-3);
  }
  dl {
    margin: 0;
  }
  .row {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 3px 0;
  }
  dt {
    flex: 0 0 118px;
    display: flex;
    gap: 3px;
    justify-content: flex-end;
  }
  dd {
    margin: 0;
    flex: 1;
    font-family: "Inter", sans-serif;
    font-size: 12px;
    color: var(--ink-2);
    line-height: 1.4;
  }
  kbd {
    font-family: "JetBrains Mono", monospace;
    font-size: 10px;
    background: var(--chrome-2);
    border: 1px solid var(--chrome-edge);
    border-bottom-width: 2px;
    border-radius: 4px;
    padding: 2px 5px;
    color: var(--ink-2);
    white-space: nowrap;
  }
  .x:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
