<script lang="ts">
  export interface Tab {
    id: string;
    name: string;
    pin: "pinned-l" | "pinned-r" | null;
    dirty: boolean;
  }

  interface Props {
    tabs: Tab[];
    activeId: string;
  }
  let { tabs, activeId }: Props = $props();
</script>

<div class="sk-tabs">
  {#each tabs as tab (tab.id)}
    <div class="sk-tab {tab.id === activeId ? 'active' : ''} {tab.pin ?? ''}">
      {#if tab.pin === "pinned-l"}
        <span class="pin-ind">
          <svg
            width="10"
            height="10"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
          >
            <path d="M2 2v8M5 4l4-2v8L5 8z" fill="currentColor" fill-opacity="0.4" />
          </svg>
        </span>
      {:else if tab.pin === "pinned-r"}
        <span class="pin-ind">
          <svg
            width="10"
            height="10"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
          >
            <path d="M10 2v8M7 4l-4-2v8l4-2z" fill="currentColor" fill-opacity="0.4" />
          </svg>
        </span>
      {:else}
        <span class="pin-ind">
          <svg
            width="10"
            height="10"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            stroke-width="1.3"
          >
            <path d="M2.5 1.5h5l2 2v7h-7zM7 1.5v2h2" />
          </svg>
        </span>
      {/if}
      <span class="name">{tab.name}</span>
      {#if tab.dirty}<span class="dirty"></span>{/if}
      <span class="x">×</span>
    </div>
  {/each}
</div>
