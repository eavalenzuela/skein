<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, drawSelection, lineNumbers } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { skeinEditorTheme, skeinHighlighting } from "./theme.js";
  import { skeinLivePreview } from "./livePreview.js";
  import { wikilinkAutocomplete } from "./wikilinkComplete.js";

  interface Props {
    doc: string;
    onChange: (next: string) => void;
  }
  let { doc, onChange }: Props = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = null;
  let lastDoc = "";

  onMount(() => {
    const initial = doc;
    lastDoc = initial;
    view = new EditorView({
      parent: container,
      state: EditorState.create({
        doc: initial,
        extensions: [
          history(),
          drawSelection(),
          // gutter off — pages are paper, not code editors
          lineNumbers({ formatNumber: () => "" }),
          markdown(),
          syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
          skeinHighlighting,
          skeinLivePreview,
          wikilinkAutocomplete,
          skeinEditorTheme,
          keymap.of([...defaultKeymap, ...historyKeymap]),
          EditorView.lineWrapping,
          EditorView.updateListener.of((u) => {
            if (u.docChanged) {
              const next = u.state.doc.toString();
              if (next !== lastDoc) {
                lastDoc = next;
                onChange(next);
              }
            }
          }),
        ],
      }),
    });
  });

  onDestroy(() => {
    view?.destroy();
  });

  $effect(() => {
    // External doc change (file watcher reload). Replace buffer only if the
    // incoming doc differs from what we last reported up.
    if (view && doc !== lastDoc) {
      lastDoc = doc;
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: doc },
      });
    }
  });
</script>

<div class="editor-host" bind:this={container}></div>

<style>
  .editor-host {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .editor-host :global(.cm-editor) {
    flex: 1;
    height: 100%;
  }
  .editor-host :global(.cm-editor.cm-focused) {
    outline: none;
  }
  /* Hide the no-number gutter we added solely to suppress CodeMirror's default. */
  .editor-host :global(.cm-gutters) {
    display: none;
  }
</style>
