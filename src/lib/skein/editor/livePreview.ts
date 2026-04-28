// Skein live-preview extension for CodeMirror 6.
//
// Pragmatic Phase 3 cut:
//   - Hide ATX heading marks (#, ##, …) when the cursor is not on that line.
//   - Hide emphasis / strong / code / link delimiters when the cursor is
//     not touching them, so bold/italic/links read like rendered output.
//   - Style [[wikilinks]] with the accent treatment.
//   - Decorate `> ` blockquote lines and hide the marker off-line.
//   - Decorate fenced code-block lines with a tinted background.
//   - Style #tag tokens.
//
// Obsidian-faithful WYSIWYG (table preview, list bullets, etc.) can be
// added incrementally on top.

import { syntaxTree } from "@codemirror/language";
import {
  Decoration,
  EditorView,
  ViewPlugin,
  type DecorationSet,
  type ViewUpdate,
} from "@codemirror/view";
import { type EditorState, RangeSetBuilder } from "@codemirror/state";

const hideMark = Decoration.replace({});
const wikilinkMark = Decoration.mark({ class: "sk-wikilink" });
const tagMark = Decoration.mark({ class: "sk-tag" });
const blockquoteLine = Decoration.line({ class: "sk-blockquote" });
const codeBlockLine = Decoration.line({ class: "sk-code-block" });

function lineOfPos(state: EditorState, pos: number): number {
  return state.doc.lineAt(pos).number;
}

function selectionTouches(state: EditorState, from: number, to: number): boolean {
  for (const range of state.selection.ranges) {
    if (range.from <= to && range.to >= from) return true;
  }
  return false;
}

function selectionOnLine(state: EditorState, pos: number): boolean {
  const line = lineOfPos(state, pos);
  for (const range of state.selection.ranges) {
    if (lineOfPos(state, range.from) === line || lineOfPos(state, range.to) === line) {
      return true;
    }
  }
  return false;
}

interface RangedDeco {
  from: number;
  to: number;
  deco: Decoration;
  // Tie-breaker for stable ordering: line decorations sort before mark/replace
  // when starting at the same position.
  rank: number;
}

function buildDecorations(view: EditorView): DecorationSet {
  const state = view.state;
  const tree = syntaxTree(state);
  const ranges: RangedDeco[] = [];

  // Wikilinks + tags via regex over visible text.
  for (const { from, to } of view.visibleRanges) {
    const text = state.doc.sliceString(from, to);

    const wikiRe = /\[\[[^\]\n]+\]\]/g;
    let m: RegExpExecArray | null;
    while ((m = wikiRe.exec(text)) !== null) {
      const start = from + m.index;
      const end = start + m[0].length;
      ranges.push({ from: start, to: end, deco: wikilinkMark, rank: 1 });
    }

    const tagRe = /(^|\s)(#[A-Za-z][\w-]*)/g;
    while ((m = tagRe.exec(text)) !== null) {
      const tagStart = from + m.index + m[1].length;
      const tagEnd = tagStart + m[2].length;
      ranges.push({ from: tagStart, to: tagEnd, deco: tagMark, rank: 1 });
    }
  }

  tree.iterate({
    enter: (node) => {
      switch (node.name) {
        case "ATXHeading1":
        case "ATXHeading2":
        case "ATXHeading3":
        case "ATXHeading4":
        case "ATXHeading5":
        case "ATXHeading6": {
          if (selectionOnLine(state, node.from)) return;
          const lineText = state.doc.sliceString(node.from, node.to);
          const mm = lineText.match(/^(#+\s*)/);
          if (mm) {
            ranges.push({
              from: node.from,
              to: node.from + mm[1].length,
              deco: hideMark,
              rank: 2,
            });
          }
          break;
        }
        case "Blockquote": {
          const startLine = state.doc.lineAt(node.from).number;
          const endLine = state.doc.lineAt(node.to).number;
          for (let l = startLine; l <= endLine; l++) {
            const line = state.doc.line(l);
            ranges.push({ from: line.from, to: line.from, deco: blockquoteLine, rank: 0 });
            if (!selectionOnLine(state, line.from)) {
              const lm = line.text.match(/^>\s?/);
              if (lm) {
                ranges.push({
                  from: line.from,
                  to: line.from + lm[0].length,
                  deco: hideMark,
                  rank: 2,
                });
              }
            }
          }
          break;
        }
        case "FencedCode": {
          const startLine = state.doc.lineAt(node.from).number;
          const endLine = state.doc.lineAt(node.to).number;
          for (let l = startLine; l <= endLine; l++) {
            const line = state.doc.line(l);
            ranges.push({ from: line.from, to: line.from, deco: codeBlockLine, rank: 0 });
          }
          break;
        }
        case "EmphasisMark":
        case "StrongMark":
        case "CodeMark":
        case "LinkMark": {
          if (selectionTouches(state, node.from - 1, node.to + 1)) return;
          ranges.push({ from: node.from, to: node.to, deco: hideMark, rank: 2 });
          break;
        }
      }
    },
  });

  ranges.sort((a, b) => a.from - b.from || a.rank - b.rank || a.to - b.to);
  const builder = new RangeSetBuilder<Decoration>();
  for (const r of ranges) builder.add(r.from, r.to, r.deco);
  return builder.finish();
}

export const skeinLivePreview = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = buildDecorations(view);
    }
    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet || update.viewportChanged) {
        this.decorations = buildDecorations(update.view);
      }
    }
  },
  {
    decorations: (v) => v.decorations,
  },
);
