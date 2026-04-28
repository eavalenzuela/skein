<script lang="ts">
  interface Props {
    x: number;
    y: number;
    text: string;
  }
  let { x, y, text }: Props = $props();
</script>

<div class="sk-drag-preview" style:left={`${x + 12}px`} style:top={`${y + 4}px`}>
  “{text}”
</div>
<div class="sk-cursor-icon" style:left={`${x - 4}px`} style:top={`${y - 4}px`}>
  <svg width="20" height="22" viewBox="0 0 20 22" fill="currentColor">
    <path
      d="M2 1.5v17l5-4 3 6.2 2.6-1.2L9.5 13H16z"
      stroke="black"
      stroke-width="0.8"
      stroke-linejoin="round"
    />
  </svg>
</div>
