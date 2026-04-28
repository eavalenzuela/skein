// Skein editor theme — pulls from the same OKLCH tokens used in the
// page-card mockup so the editor surface looks like a sheet of paper,
// not generic CodeMirror chrome.

import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

export const skeinEditorTheme = EditorView.theme(
  {
    "&": {
      color: "var(--ink)",
      backgroundColor: "transparent",
      fontFamily: 'var(--page-font, "Source Serif 4"), Georgia, serif',
      fontSize: "14.5px",
      height: "100%",
    },
    ".cm-scroller": {
      fontFamily: 'var(--page-font, "Source Serif 4"), Georgia, serif',
      lineHeight: "1.65",
      overflow: "auto",
    },
    ".cm-content": {
      padding: "38px 56px 42px",
      caretColor: "var(--accent)",
      maxWidth: "none",
    },
    ".cm-line": {
      padding: "0",
    },
    ".cm-cursor, .cm-dropCursor": {
      borderLeftColor: "var(--accent)",
      borderLeftWidth: "2px",
    },
    "&.cm-focused .cm-cursor": {
      borderLeftColor: "var(--accent)",
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": {
      backgroundColor: "var(--accent-soft)",
    },
    ".cm-activeLine": {
      backgroundColor: "transparent",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "transparent",
    },
    ".cm-gutters": {
      display: "none",
    },
    ".cm-tooltip": {
      background: "var(--chrome-2)",
      border: "1px solid var(--chrome-edge)",
      color: "var(--ink)",
    },
    // Live-preview decoration classes (added by livePreview.ts).
    ".sk-h1": {
      fontFamily: '"Source Serif 4", Georgia, serif',
      fontWeight: "600",
      fontSize: "22px",
      letterSpacing: "-0.005em",
    },
    ".sk-h2": {
      fontFamily: '"Source Serif 4", Georgia, serif',
      fontWeight: "600",
      fontSize: "16px",
    },
    ".sk-h3": {
      fontFamily: '"Source Serif 4", Georgia, serif',
      fontWeight: "600",
      fontSize: "14px",
      color: "var(--ink-2)",
    },
    ".sk-h4, .sk-h5, .sk-h6": {
      fontFamily: '"Source Serif 4", Georgia, serif',
      fontWeight: "600",
      fontSize: "13px",
      color: "var(--ink-2)",
    },
    ".sk-mark": {
      color: "var(--ink-4)",
      fontFamily: '"JetBrains Mono", monospace',
      fontSize: "0.88em",
    },
    ".sk-blockquote": {
      borderLeft: "2px solid var(--accent-edge)",
      paddingLeft: "14px",
      color: "var(--ink-2)",
      fontStyle: "italic",
    },
    ".sk-inline-code": {
      fontFamily: '"JetBrains Mono", monospace',
      fontSize: "12px",
      background: "oklch(from var(--page) calc(l - 0.04) c h)",
      padding: "1px 5px",
      borderRadius: "3px",
    },
    ".sk-code-block": {
      fontFamily: '"JetBrains Mono", monospace',
      fontSize: "12px",
      background: "oklch(from var(--page) calc(l - 0.04) c h)",
    },
    ".sk-wikilink": {
      color: "var(--accent)",
      borderBottom: "1px dashed var(--accent-edge)",
    },
    ".sk-link": {
      color: "var(--accent)",
    },
    ".sk-tag": {
      color: "var(--accent)",
      fontFamily: '"JetBrains Mono", monospace',
      fontSize: "12px",
    },
    ".sk-em": {
      fontStyle: "italic",
    },
    ".sk-strong": {
      fontWeight: "600",
    },
    ".sk-strike": {
      textDecoration: "line-through",
      color: "var(--ink-3)",
    },
    ".sk-hr": {
      borderTop: "1px solid var(--page-edge)",
      display: "inline-block",
      width: "100%",
    },
  },
  { dark: true },
);

// Highlighting for the markdown grammar nodes that aren't covered by our
// live-preview decorations (mostly inline emphasis tokens we leave visible
// when the cursor is on the line so the user can edit them).
export const skeinHighlighting = syntaxHighlighting(
  HighlightStyle.define([
    { tag: t.heading1, class: "sk-h1" },
    { tag: t.heading2, class: "sk-h2" },
    { tag: t.heading3, class: "sk-h3" },
    { tag: t.heading4, class: "sk-h4" },
    { tag: t.heading5, class: "sk-h5" },
    { tag: t.heading6, class: "sk-h6" },
    { tag: t.strong, class: "sk-strong" },
    { tag: t.emphasis, class: "sk-em" },
    { tag: t.strikethrough, class: "sk-strike" },
    { tag: t.link, class: "sk-link" },
    { tag: t.url, class: "sk-link" },
    { tag: t.monospace, class: "sk-inline-code" },
    { tag: t.processingInstruction, class: "sk-mark" },
    { tag: t.contentSeparator, class: "sk-hr" },
  ]),
);
