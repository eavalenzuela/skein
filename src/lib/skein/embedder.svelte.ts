// Embedder model state — backs the "Embeddings" row in DevControls and
// (later) the proper Settings UI in Phase 6.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  embeddingModelStatus,
  downloadEmbeddingModel,
  type EmbeddingModelStatus,
} from "./vault.js";

export const embedderState: {
  status: EmbeddingModelStatus | null;
  busy: boolean;
  error: string | null;
} = $state({
  status: null,
  busy: false,
  error: null,
});

let unlisten: UnlistenFn | null = null;

export async function bootstrap() {
  try {
    embedderState.status = await embeddingModelStatus();
  } catch (e) {
    embedderState.error = String(e);
  }
  if (!unlisten) {
    unlisten = await listen<{ state: string; name?: string }>("embedding-model", (ev) => {
      if (ev.payload.state === "ready") {
        embedderState.busy = false;
        embeddingModelStatus()
          .then((s) => {
            embedderState.status = s;
          })
          .catch(() => {});
      } else if (ev.payload.state === "downloading") {
        embedderState.busy = true;
        embedderState.error = null;
      }
    });
  }
}

export async function downloadModel() {
  embedderState.busy = true;
  embedderState.error = null;
  try {
    embedderState.status = await downloadEmbeddingModel();
  } catch (e) {
    embedderState.error = String(e);
  } finally {
    embedderState.busy = false;
  }
}
