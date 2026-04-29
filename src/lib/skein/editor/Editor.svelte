<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Compartment, EditorState } from "@codemirror/state";
  import { EditorView, keymap, drawSelection, lineNumbers } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { skeinEditorTheme, skeinHighlighting } from "./theme.js";
  import { skeinLivePreview, pageContextExtension } from "./livePreview.js";
  import { wikilinkAutocomplete } from "./wikilinkComplete.js";
  import { saveAttachment, saveAttachmentFromPath } from "../vault.js";
  import { vaultState } from "../vault.svelte.js";
  import { readImage } from "@tauri-apps/plugin-clipboard-manager";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  interface Props {
    doc: string;
    relPath: string;
    onChange: (next: string) => void;
  }
  let { doc, relPath, onChange }: Props = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = null;
  let lastDoc = "";
  // Snapshot of relPath kept in sync via $effect, so paste/drop handlers
  // (which run outside the reactive scope) see the current page.
  let currentRelPath = $state(relPath);
  const pageCtxCompartment = new Compartment();

  function pageCtxFor(rp: string) {
    const root = vaultState.vault?.root ?? "";
    return pageContextExtension({ vaultRoot: root, pageRelPath: rp });
  }

  // Pull a sane file extension from a MIME type or filename. Empty string
  // falls back to "bin" on the Rust side.
  function extFromMime(mime: string): string {
    const slash = mime.indexOf("/");
    return slash === -1 ? "" : mime.slice(slash + 1);
  }
  function extFromName(name: string): string {
    const dot = name.lastIndexOf(".");
    return dot === -1 ? "" : name.slice(dot + 1);
  }

  // Resolve a vault-relative attachment path to a path relative to the
  // current page's folder, so the inserted markdown ref stays valid if the
  // book is later moved as a unit.
  function relativeToPage(attachmentRel: string): string {
    const slash = currentRelPath.lastIndexOf("/");
    if (slash === -1) return attachmentRel;
    const folder = currentRelPath.slice(0, slash + 1);
    return attachmentRel.startsWith(folder)
      ? attachmentRel.slice(folder.length)
      : attachmentRel;
  }

  function insertMarkdown(view: EditorView, md: string) {
    const { from, to } = view.state.selection.main;
    view.dispatch({
      changes: { from, to, insert: md },
      selection: { anchor: from + md.length },
    });
  }

  async function insertAttachment(view: EditorView, file: File | Blob, fallbackName: string) {
    const buf = new Uint8Array(await file.arrayBuffer());
    if (buf.length === 0) return;
    const name = (file as File).name || fallbackName;
    const ext = extFromName(name) || extFromMime(file.type);
    const rel = await saveAttachment(currentRelPath, ext, buf);
    const refPath = relativeToPage(rel);
    const altBase = (file as File).name
      ? (file as File).name.replace(/\.[^.]+$/, "")
      : "image";
    insertMarkdown(view, `![${altBase}](${refPath})`);
  }

  async function tryClipboardImagePaste(view: EditorView): Promise<boolean> {
    let img;
    try {
      img = await readImage();
    } catch {
      return false;
    }
    let rgba: Uint8Array;
    let size: { width: number; height: number };
    try {
      rgba = await img.rgba();
      size = await img.size();
    } catch {
      return false;
    }
    if (!rgba.length || !size.width || !size.height) return false;
    const canvas = new OffscreenCanvas(size.width, size.height);
    const ctx = canvas.getContext("2d");
    if (!ctx) return false;
    const imageData = new ImageData(new Uint8ClampedArray(rgba), size.width, size.height);
    ctx.putImageData(imageData, 0, 0);
    const blob = await canvas.convertToBlob({ type: "image/png" });
    const buf = new Uint8Array(await blob.arrayBuffer());
    const rel = await saveAttachment(currentRelPath, "png", buf);
    insertMarkdown(view, `![pasted-image](${relativeToPage(rel)})`);
    return true;
  }

  async function insertAttachmentFromPath(view: EditorView, srcPath: string) {
    const rel = await saveAttachmentFromPath(currentRelPath, srcPath);
    const refPath = relativeToPage(rel);
    const slash = srcPath.lastIndexOf("/");
    const baseName = (slash === -1 ? srcPath : srcPath.slice(slash + 1)).replace(
      /\.[^.]+$/,
      "",
    );
    insertMarkdown(view, `![${baseName || "image"}](${refPath})`);
  }

  function isImage(item: { type: string }): boolean {
    return item.type.startsWith("image/");
  }

  // Image extensions we treat as attachments when dragged from the OS as a
  // text/uri-list (no MIME info available at that point).
  const IMG_EXTS = new Set(["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "avif"]);
  function isImageExt(path: string): boolean {
    const ext = extFromName(path).toLowerCase();
    return IMG_EXTS.has(ext);
  }


  // On Linux WebKitGTK, file drops into the webview navigate the page
  // unless Tauri intercepts at the OS level (the `dragDropEnabled: true`
  // default). We listen for Tauri's native drag-drop events here and route
  // the file paths into the editor whose container the drop lands in.
  let unlistenDrop: UnlistenFn | null = null;

  onMount(() => {
    void (async () => {
      unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type !== "drop") return;
        if (!view || !container) return;
        const rect = container.getBoundingClientRect();
        // Tauri's drop position is in physical pixels relative to the
        // window. The webview's inner CSS pixel coords match the window's
        // CSS coords here on Linux, so use them directly.
        const { x, y } = event.payload.position;
        const inside = x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
        if (!inside) return;
        const paths = event.payload.paths.filter(isImageExt);
        if (paths.length === 0) return;
        const pos = view.posAtCoords({ x, y });
        if (pos != null) view.dispatch({ selection: { anchor: pos } });
        const v = view;
        (async () => {
          for (const p of paths) await insertAttachmentFromPath(v, p);
        })();
      });
    })();
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
          pageCtxCompartment.of(pageCtxFor(relPath)),
          skeinLivePreview,
          wikilinkAutocomplete,
          skeinEditorTheme,
          keymap.of([...defaultKeymap, ...historyKeymap]),
          EditorView.lineWrapping,
          EditorView.domEventHandlers({
            paste(event, v) {
              // Try the browser clipboard path first (works for images
              // pasted from web apps that put image data in clipboardData).
              const cd = event.clipboardData;
              const files: File[] = [];
              if (cd?.files?.length) {
                for (const f of cd.files) if (isImage(f)) files.push(f);
              }
              if (files.length === 0 && cd?.items) {
                for (const item of cd.items) {
                  if (item.kind === "file" && isImage(item)) {
                    const f = item.getAsFile();
                    if (f) files.push(f);
                  }
                }
              }
              if (files.length > 0) {
                event.preventDefault();
                (async () => {
                  for (const f of files) {
                    await insertAttachment(v, f, "pasted-image");
                  }
                })();
                return true;
              }
              // WebKitGTK often doesn't surface screenshot images through
              // clipboardData. Capture any text payload synchronously, then
              // probe the OS clipboard via Tauri — if it has an image we
              // insert it; otherwise we replay the captured text.
              const fallbackText = cd?.getData("text/plain") ?? "";
              event.preventDefault();
              (async () => {
                const inserted = await tryClipboardImagePaste(v).catch(() => false);
                if (!inserted && fallbackText) {
                  insertMarkdown(v, fallbackText);
                }
              })();
              return true;
            },
            dragover(event) {
              const dt = event.dataTransfer;
              if (!dt) return false;
              if (Array.from(dt.types ?? []).includes("application/x-skein-chat")) {
                event.preventDefault();
                if (dt) dt.dropEffect = "copy";
                return true;
              }
              return false;
            },
            drop(event, v) {
              const dt = event.dataTransfer;
              if (!dt) return false;
              const chatText = dt.getData("application/x-skein-chat");
              if (!chatText) return false;
              event.preventDefault();
              const pos = v.posAtCoords({ x: event.clientX, y: event.clientY });
              if (pos != null) {
                v.dispatch({
                  changes: { from: pos, insert: chatText },
                  selection: { anchor: pos + chatText.length },
                });
              } else {
                insertMarkdown(v, chatText);
              }
              return true;
            },
          }),
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
    unlistenDrop?.();
    view?.destroy();
  });

  $effect(() => {
    currentRelPath = relPath;
    if (view) {
      view.dispatch({
        effects: pageCtxCompartment.reconfigure(pageCtxFor(relPath)),
      });
    }
  });

  $effect(() => {
    // Reconfigure when the vault root becomes known (or changes).
    const root = vaultState.vault?.root;
    if (view && root !== undefined) {
      view.dispatch({
        effects: pageCtxCompartment.reconfigure(pageCtxFor(currentRelPath)),
      });
    }
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
