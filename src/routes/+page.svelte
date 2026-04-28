<script lang="ts">
  import SkeinWindow from "$lib/skein/SkeinWindow.svelte";
  import DevControls from "$lib/skein/DevControls.svelte";
  import { tweaks } from "$lib/skein/tweaks.svelte.js";
</script>

<div class="app-frame">
  <SkeinWindow
    theme={tweaks.theme}
    shelfStyle={tweaks.shelfStyle}
    sidebar={tweaks.sidebar}
    scenario={tweaks.scenario}
    pageFont={tweaks.pageFont}
    vault="Field Notes"
  />
</div>

<DevControls />

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    height: 100%;
    background: #1a1a1c;
    overflow: hidden;
  }
  :global(*) {
    box-sizing: border-box;
  }
  .app-frame {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }
</style>
