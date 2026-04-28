<script lang="ts">
  interface Props {
    withSelection?: boolean;
    streaming?: boolean;
  }
  let { withSelection = false, streaming = false }: Props = $props();
</script>

<div class="sk-chat">
  <div class="sk-msg user">
    Re-reading <i>The Peregrine</i>. Baker has a sentence I can't shake — something about how the
    hardest thing to see is what's really there. Where else does that idea show up in what I've been
    reading lately?
  </div>

  <div class="sk-msg asst">
    <div class="who"><span class="ico">A</span> Haiku 4.5 · current + related</div>
    <p>Three threads in your vault pull on the same idea, just from different angles:</p>
    <p>
      <b>Annie Dillard</b>, in <code>Reading Notes/Tinker Creek.md</code>, has the line you
      bracketed last March: looking is an act of attention before it is an act of perception. Same
      shape as Baker — the seeing is the practice, not the prize.
    </p>
    <p>
      <b>Iain McGilchrist</b>, in your March 6 daily, on the difference between <i>knowing</i> a
      tree and <i>seeing</i> it. You wrote "this is the falcon argument" in the margin.
    </p>
    <p>
      And maybe a stretch: your note on <code>Ideas/marginalia-mode.md</code> — "what is unread is not
      yet there" — is the same claim turned toward writing instead of watching.
    </p>
  </div>

  <div class="sk-msg user">Pull the Dillard sentence so I can drop it next to the Baker one.</div>

  <div class="sk-msg asst">
    <div class="who"><span class="ico">A</span> Haiku 4.5 · current + related</div>
    <p>Here it is, exactly as you transcribed it:</p>
    <p>
      {#if withSelection}
        <span class="chat-selection" data-chat-selection
          >“Seeing is of course very much a matter of verbalization. Unless I call my attention to
          what passes before my eyes, I simply won't see it.”</span
        >
      {:else}
        <span
          >“Seeing is of course very much a matter of verbalization. Unless I call my attention to
          what passes before my eyes, I simply won't see it.”</span
        >
      {/if}
    </p>
    <p>
      Source: <code>Reading Notes/Tinker Creek.md</code>, line 47. It sits a paragraph above the
      Baker quote in your{#if streaming}<span class="streaming-dot"></span>{:else}
        current note.{/if}
    </p>
  </div>
</div>
