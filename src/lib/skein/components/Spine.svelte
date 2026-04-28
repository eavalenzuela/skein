<script lang="ts">
  import type { Theme } from "../tweaks.svelte.js";

  interface Props {
    title: string;
    h: number;
    hue: number;
    shade: number;
    active: boolean;
    narrow?: boolean;
    theme: Theme;
  }
  let { title, h, hue, shade, active, narrow = false, theme }: Props = $props();

  let bg = $derived(`oklch(${shade.toFixed(2)} 0.025 ${50 + (hue % 30) - 15})`);
  let band = $derived(`oklch(${theme === "light" ? 0.55 : 0.5} 0.085 ${hue})`);
</script>

<div
  class="sk-spine"
  class:active
  style:--spine-h={`${h}px`}
  style:--spine-w={`${narrow ? 22 : 28}px`}
  style:--spine-bg={bg}
  style:--spine-band={band}
>
  <div class="band"></div>
  <div class="band-2"></div>
  <div class="title below-band">{title}</div>
</div>
