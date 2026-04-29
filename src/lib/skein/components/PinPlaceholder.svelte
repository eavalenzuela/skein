<script lang="ts">
  interface Props {
    side: "left" | "right";
  }
  let { side }: Props = $props();
</script>

<div class="pin-placeholder">
  <div class="card">
    <div class="title">{side === "left" ? "Left" : "Right"} pane is empty</div>
    <p>Click any page or drag one here to open it on this side.</p>
  </div>
</div>

<style>
  .pin-placeholder {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--page);
    color: var(--ink-3);
    font-family: "Inter", system-ui, sans-serif;
    border: 1px dashed var(--page-edge);
    border-radius: 4px;
    margin: 6px;
  }
  .card {
    text-align: center;
    padding: 24px;
    max-width: 280px;
  }
  .title {
    font-family: "Source Serif 4", Georgia, serif;
    font-size: 15px;
    font-weight: 600;
    color: var(--ink-2);
    margin-bottom: 6px;
  }
  p {
    font-size: 12.5px;
    line-height: 1.55;
    margin: 0;
  }
</style>
