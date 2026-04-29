<script lang="ts" module>
  export interface MenuItem {
    label: string;
    action: () => void;
    danger?: boolean;
    separator?: boolean;
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";

  interface Props {
    x: number;
    y: number;
    items: MenuItem[];
    onclose: () => void;
  }
  let { x, y, items, onclose }: Props = $props();

  let menuEl: HTMLDivElement;
  let active = $state(firstSelectable(items));

  function firstSelectable(list: MenuItem[]): number {
    for (let i = 0; i < list.length; i++) if (!list[i].separator) return i;
    return 0;
  }
  function step(from: number, dir: 1 | -1): number {
    const n = items.length;
    let i = from;
    for (let k = 0; k < n; k++) {
      i = (i + dir + n) % n;
      if (!items[i].separator) return i;
    }
    return from;
  }

  // Clamp the menu inside the viewport so it doesn't render off-edge when
  // the user right-clicks near the bottom or right of the window.
  let clamped = $state({ x, y });
  onMount(() => {
    const rect = menuEl.getBoundingClientRect();
    const maxX = window.innerWidth - rect.width - 8;
    const maxY = window.innerHeight - rect.height - 8;
    clamped = { x: Math.min(x, Math.max(8, maxX)), y: Math.min(y, Math.max(8, maxY)) };
    menuEl.focus();

    function onDocClick(e: MouseEvent) {
      if (menuEl && !menuEl.contains(e.target as Node)) onclose();
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        onclose();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        active = step(active, 1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        active = step(active, -1);
      } else if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        const it = items[active];
        if (it && !it.separator) {
          it.action();
          onclose();
        }
      } else if (e.key === "Home") {
        e.preventDefault();
        active = firstSelectable(items);
      } else if (e.key === "End") {
        e.preventDefault();
        active = step(items.length, -1);
      }
    }
    document.addEventListener("mousedown", onDocClick, true);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDocClick, true);
      document.removeEventListener("keydown", onKey);
    };
  });
</script>

<div
  bind:this={menuEl}
  class="sk-ctxmenu"
  style:left={`${clamped.x}px`}
  style:top={`${clamped.y}px`}
  role="menu"
  tabindex="-1"
>
  {#each items as item, i (i)}
    {#if item.separator}
      <div class="sep"></div>
    {:else}
      <button
        class="item"
        class:danger={item.danger}
        class:active={i === active}
        onmouseenter={() => (active = i)}
        onclick={() => {
          item.action();
          onclose();
        }}
        role="menuitem"
      >
        {item.label}
      </button>
    {/if}
  {/each}
</div>

<style>
  .sk-ctxmenu {
    position: fixed;
    background: var(--chrome-2, #2a2a2c);
    border: 1px solid var(--chrome-edge, #3a3a3d);
    border-radius: 6px;
    padding: 4px 0;
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.45);
    z-index: 1000;
    min-width: 160px;
    font-family: "Inter", system-ui, sans-serif;
    font-size: 12px;
  }
  .item {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: 0;
    padding: 7px 14px;
    color: var(--ink, #e6e6e6);
    cursor: pointer;
    font: inherit;
  }
  .item:hover,
  .item.active {
    background: var(--accent-soft, rgba(200, 160, 80, 0.18));
  }
  .sk-ctxmenu:focus {
    outline: none;
  }
  .item.danger {
    color: oklch(0.7 0.16 25);
  }
  .item.danger:hover {
    background: oklch(0.65 0.18 25 / 0.18);
  }
  .sep {
    height: 1px;
    background: var(--chrome-edge, #3a3a3d);
    margin: 4px 0;
  }
</style>
