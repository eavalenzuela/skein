<script lang="ts">
  interface Props {
    withCaret?: boolean;
    caretAt?: "shelf" | "editor" | null;
  }
  let { withCaret = false, caretAt = null }: Props = $props();
</script>

<div class="sk-page-inner">
  <h1>Tuesday, 21 April</h1>
  <div class="muted">Daily · 7 min read · 4 backlinks</div>

  <h2>Morning</h2>
  <p>
    Slept poorly, but the Sycamore at the end of the lane was throwing enough shade by 7 to read
    outside. Finished the last chapter of <!-- svelte-ignore a11y_missing_attribute -->
    <a class="wiki"> [[The Peregrine]]</a> — the section on the cliff hunts. Baker writes about watching
    like it's a craft you have to practice into existence.
  </p>

  <blockquote>
    "To be recognised and accepted by a peregrine you must learn to fit into a landscape and use it
    to your advantage."
  </blockquote>

  <h2>Skein, mid-build</h2>
  <p>
    Spent the afternoon on the bookshelf metaphor — the shelf needs to feel like a
    {#if withCaret && caretAt === "shelf"}<span class="sk-caret insert"></span>{/if}
    place, not a sidebar. Right now the spines look too uniform. Maybe vary the heights more, or let one
    or two lean.
  </p>

  <p>Decisions today:</p>
  <ul>
    <li>Keep the chat panel collapsible, not removable.</li>
    <li>
      <span class="md-syntax">**</span>Drag-to-insert<span class="md-syntax">**</span> beats auto-saving
      the transcript. Fewer surprises.
    </li>
    <li>Folio at the start of the shelf, not the end.</li>
  </ul>

  <h2>Tomorrow</h2>
  <p>
    Pick up <!-- svelte-ignore a11y_missing_attribute -->
    <a class="wiki">[[Project Ideas]]</a> — the note about the marginalia mode. Also: water the
    rosemary{#if !withCaret}<span class="sk-caret"
      ></span>{/if}{#if withCaret && caretAt === "editor"}<span class="sk-caret insert"
      ></span>{/if}.
  </p>

  <p class="muted">
    <span class="tag">#daily</span> &nbsp;
    <span class="tag">#skein</span> &nbsp;
    <span class="tag">#reading</span>
  </p>
</div>
