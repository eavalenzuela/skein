// CodeMirror 6 autocomplete for [[wikilinks]] backed by the live page-title
// cache. Pulls candidates from titlesState; the cache stays fresh via the
// "vault-changed" listener in titles.svelte.ts.

import {
  autocompletion,
  type Completion,
  type CompletionContext,
  type CompletionResult,
} from "@codemirror/autocomplete";
import { titlesState } from "../titles.svelte.js";

function source(context: CompletionContext): CompletionResult | null {
  // Match `[[` followed by anything except `]` or newline up to the cursor.
  const before = context.matchBefore(/\[\[[^\]\n]*/);
  if (!before) return null;
  if (before.from === before.to && !context.explicit) return null;

  const queryRaw = before.text.slice(2);
  const query = queryRaw.toLowerCase().trim();

  const items: Completion[] = titlesState.items
    .filter((t) => !query || t.title.toLowerCase().includes(query))
    .slice(0, 30)
    .map((t) => ({
      label: t.title,
      detail: t.rel_path,
      type: "text",
      apply: t.title + "]]",
      boost: t.title.toLowerCase().startsWith(query) ? 1 : 0,
    }));

  return {
    from: before.from + 2,
    options: items,
    validFor: /^[^\]\n]*$/,
  };
}

export const wikilinkAutocomplete = autocompletion({
  override: [source],
  defaultKeymap: true,
  closeOnBlur: true,
});
