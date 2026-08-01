<script lang="ts">
  import { toastState, dismissToast, type Toast } from "../toasts.svelte.js";

  async function runAction(t: Toast) {
    const action = t.action;
    dismissToast(t.id);
    if (action) await action.run();
  }
</script>

<!-- Bottom-left so it never covers the chat sidebar or the tab row. -->
<div class="toasts" role="region" aria-label="Notifications">
  {#each toastState.items as t (t.id)}
    <div class="toast {t.kind}" role={t.kind === "error" ? "alert" : "status"}>
      <div class="body">
        <p class="msg">{t.message}</p>
        {#if t.detail}
          <p class="detail">{t.detail}</p>
        {/if}
      </div>
      {#if t.action}
        <button class="act" onclick={() => runAction(t)}>{t.action.label}</button>
      {/if}
      <button class="x" onclick={() => dismissToast(t.id)} aria-label="Dismiss notification">
        ×
      </button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: absolute;
    left: 16px;
    bottom: 16px;
    z-index: 900;
    display: flex;
    flex-direction: column-reverse;
    gap: 8px;
    max-width: min(420px, 60vw);
    pointer-events: none;
  }
  .toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 10px 10px 12px;
    border-radius: 8px;
    background: var(--chrome-2);
    border: 1px solid var(--chrome-edge);
    border-left-width: 3px;
    box-shadow: 0 10px 28px -12px oklch(0 0 0 / 0.55);
    animation: rise 0.16s ease-out;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .toast {
      animation: none;
    }
  }
  .toast.error {
    border-left-color: var(--danger);
    background: color-mix(in oklab, var(--chrome-2) 88%, var(--danger));
  }
  .toast.success {
    border-left-color: var(--ok);
  }
  .toast.info {
    border-left-color: var(--accent);
  }
  .body {
    flex: 1;
    min-width: 0;
  }
  .msg {
    margin: 0;
    font-size: 12.5px;
    color: var(--ink);
    line-height: 1.35;
  }
  .detail {
    margin: 3px 0 0;
    font-size: 11.5px;
    color: var(--ink-3);
    line-height: 1.35;
    overflow-wrap: anywhere;
  }
  .act {
    flex: none;
    background: var(--accent-soft);
    border: 1px solid var(--accent-edge);
    color: var(--ink);
    border-radius: 5px;
    padding: 4px 9px;
    font-family: inherit;
    font-size: 11.5px;
    cursor: pointer;
  }
  .act:hover {
    background: var(--accent);
    color: var(--chrome);
  }
  .x {
    flex: none;
    background: transparent;
    border: 0;
    color: var(--ink-3);
    font-size: 15px;
    line-height: 1;
    padding: 2px 4px;
    cursor: pointer;
    border-radius: 4px;
  }
  .x:hover {
    color: var(--ink);
    background: oklch(1 0 0 / 0.07);
  }
  .act:focus-visible,
  .x:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
