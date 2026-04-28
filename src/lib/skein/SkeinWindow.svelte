<script lang="ts">
  import "./styles.css";
  import type { Theme, ShelfStyle, SidebarMode, Scenario, PageFont } from "./tweaks.svelte.js";
  import Titlebar from "./components/Titlebar.svelte";
  import Bookshelf from "./components/Bookshelf.svelte";
  import Desk from "./components/Desk.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import DragOverlay from "./components/DragOverlay.svelte";

  interface Props {
    theme?: Theme;
    shelfStyle?: ShelfStyle;
    sidebar?: SidebarMode;
    scenario?: Scenario;
    vault?: string;
    pageFont?: PageFont;
  }
  let {
    theme = "dark",
    shelfStyle = "suggestive",
    sidebar = "open",
    scenario = "populated",
    vault = "Field Notes",
    pageFont = "Source Serif 4",
  }: Props = $props();

  let isDragging = $derived(scenario === "dragging");
  let deskScenario = $derived<Scenario>(scenario === "empty" ? "empty" : "populated");
  let midClass = $derived(
    sidebar === "open"
      ? "sidebar-open"
      : sidebar === "collapsed"
        ? "sidebar-collapsed"
        : "sidebar-hidden",
  );
</script>

<div class="skein theme-{theme}" style:--page-font={pageFont}>
  <div class="win">
    <Titlebar {vault} />
    <div class="sk-body">
      <Bookshelf style={shelfStyle} {theme} />
      <div class="sk-mid {midClass}" style:position="relative">
        <Desk
          scenario={deskScenario}
          withCaret={isDragging}
          caretAt={isDragging ? "editor" : null}
        />
        <Sidebar mode={sidebar} withSelection={isDragging} streaming={false} />
        {#if isDragging}
          <DragOverlay
            x={680}
            y={350}
            text={`Seeing is of course very much a matter of verbalization. Unless I call my attention to what passes before my eyes, I simply won't see it.`}
          />
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .skein {
    width: 100%;
    height: 100%;
  }
</style>
