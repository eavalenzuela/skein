<script lang="ts">
  interface Props {
    withCaret?: boolean;
    caretAt?: "review" | "editor" | null;
  }
  let { withCaret = false, caretAt = null }: Props = $props();
</script>

<div class="sk-page-inner">
  <h1>On <em>The Peregrine</em></h1>
  <div class="muted">Reading Notes · J.A. Baker, 1967</div>

  <p>
    A book disguised as a field journal. Ten winters compressed into six months on the page. Baker
    watches falcons in Essex and slowly stops being himself.
  </p>

  <h3>What stays with me</h3>
  <p>
    The <em>looking</em>. Baker doesn't describe what a peregrine is; he describes what the act of
    seeing one does to a person. The bird is almost always at the edge of vision — a notch in the
    sky, a lift over the marsh.
    {#if withCaret && caretAt === "review"}<span class="sk-caret insert"></span>{/if}
    You are pulled along behind it.
  </p>

  <h3>Sentences I want to keep</h3>
  <blockquote>"The hardest thing of all to see is what is really there."</blockquote>

  <p>
    Compare to <!-- svelte-ignore a11y_missing_attribute -->
    <a class="wiki">[[H is for Hawk]]</a> — Macdonald grieves through the goshawk; Baker disappears into
    the falcon. Two directions out of the same forest.
  </p>

  <hr />
  <p class="muted">
    Linked from: <!-- svelte-ignore a11y_missing_attribute -->
    <a class="wiki">[[Daily/2026-04-21]]</a>
  </p>
</div>
