<script lang="ts">
  import type { Theme } from "../tweaks.svelte.js";

  interface Props {
    title: string;
    h: number;
    hue: number;
    shade: number;
    active: boolean;
    /** Override the computed width — used by callers that derive from spineWidth(name). */
    w?: number;
    narrow?: boolean;
    theme: Theme;
  }
  let { title, h, hue, shade, active, w, narrow = false, theme }: Props = $props();

  let bg = $derived(`oklch(${shade.toFixed(2)} 0.025 ${50 + (hue % 30) - 15})`);
  let band = $derived(`oklch(${theme === "light" ? 0.55 : 0.5} 0.085 ${hue})`);
  let widthPx = $derived(w ?? (narrow ? 22 : 28));

  // Fit the name to the spine. Vertical uppercase text advances at roughly
  // the font size per character; the usable run is the spine height less
  // the band clearance and padding. A wider spine wraps into columns, so
  // divide the budget by however many columns the width affords.
  let titleSize = $derived.by(() => {
    const columns = Math.max(1, Math.round(widthPx / 28));
    const usable = h - 36;
    const perColumn = Math.ceil(title.length / columns);
    const fitted = usable / (perColumn * 1.08);
    return Math.max(7.5, Math.min(10.5, Number(fitted.toFixed(2))));
  });
</script>

<div
  class="sk-spine"
  class:active
  title={title}
  style:--spine-h={`${h}px`}
  style:--spine-w={`${widthPx}px`}
  style:--spine-bg={bg}
  style:--spine-band={band}
  style:--spine-title-size={`${titleSize}px`}
>
  <div class="band"></div>
  <div class="band-2"></div>
  <div class="title below-band">{title}</div>
</div>
