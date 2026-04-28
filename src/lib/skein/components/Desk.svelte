<script lang="ts">
  import type { Scenario } from "../tweaks.svelte.js";
  import Tabs, { type Tab } from "./Tabs.svelte";
  import PageDaily from "./PageDaily.svelte";
  import PageReview from "./PageReview.svelte";
  import EmptyDesk from "./EmptyDesk.svelte";

  interface Props {
    scenario: Scenario;
    withCaret?: boolean;
    caretAt?: "shelf" | "editor" | "review" | null;
  }
  let { scenario, withCaret = false, caretAt = null }: Props = $props();

  const populatedTabs: Tab[] = [
    { id: "t1", name: "Tuesday, 21 April", pin: "pinned-l", dirty: true },
    { id: "t2", name: "On The Peregrine", pin: "pinned-r", dirty: false },
    { id: "t3", name: "brown-butter-pasta.md", pin: null, dirty: false },
  ];
</script>

{#if scenario === "empty"}
  <div class="sk-desk">
    <div class="sk-tabs-empty">No tabs open</div>
    <EmptyDesk />
  </div>
{:else}
  <div class="sk-desk">
    <Tabs tabs={populatedTabs} activeId="t1" />
    <div class="sk-surface">
      <div class="sk-page">
        <PageDaily {withCaret} caretAt={caretAt as "shelf" | "editor" | null} />
      </div>
      <div class="sk-page">
        <PageReview {withCaret} caretAt={caretAt as "review" | "editor" | null} />
      </div>
    </div>
  </div>
{/if}
