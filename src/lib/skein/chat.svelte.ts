// Phase 7 chat store. Drives the live chat sidebar; the visual chrome
// is the Sidebar component from Phase 1, just fed real data.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type ContextMode = "current" | "current+related" | "vault";

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  streaming?: boolean;
  contextMode?: ContextMode;
  contextChunks?: { rel_path: string; title: string; heading: string }[];
}

export interface ChatModel {
  id: string;
  label: string;
}

export const CHAT_MODELS: ChatModel[] = [
  { id: "claude-haiku-4-5", label: "Haiku 4.5" },
  { id: "claude-sonnet-4-6", label: "Sonnet 4.6" },
  { id: "claude-opus-4-7", label: "Opus 4.7" },
];

interface ChatEventPayload {
  turn_id: string;
  kind: "started" | "token" | "done" | "error";
  text?: string;
  error?: string;
  context?: {
    mode: string;
    current_rel_path: string | null;
    chunks: { rel_path: string; title: string; heading: string; similarity: number }[];
  };
}

export const chatState: {
  messages: ChatMessage[];
  model: string;
  contextMode: ContextMode;
  busy: boolean;
  error: string | null;
  activeTurnId: string | null;
} = $state({
  messages: [],
  model: "claude-haiku-4-5",
  contextMode: "current+related",
  busy: false,
  error: null,
  activeTurnId: null,
});

let unlisten: UnlistenFn | null = null;

export async function attachChatBus() {
  if (unlisten) return;
  unlisten = await listen<ChatEventPayload>("chat-event", (ev) => {
    const p = ev.payload;
    if (p.turn_id !== chatState.activeTurnId) return;
    const last = chatState.messages[chatState.messages.length - 1];
    if (!last || last.role !== "assistant") return;
    if (p.kind === "token" && p.text) {
      last.content += p.text;
    } else if (p.kind === "done") {
      last.streaming = false;
      if (p.context) {
        last.contextChunks = p.context.chunks.map((c) => ({
          rel_path: c.rel_path,
          title: c.title,
          heading: c.heading,
        }));
        last.contextMode = p.context.mode as ContextMode;
      }
      chatState.busy = false;
      chatState.activeTurnId = null;
    } else if (p.kind === "error") {
      last.streaming = false;
      chatState.busy = false;
      chatState.activeTurnId = null;
      chatState.error = p.error ?? "chat failed";
    }
  });
}

function newId() {
  return Math.random().toString(36).slice(2);
}

export async function send(input: string, currentRelPath: string | null) {
  const text = input.trim();
  if (!text || chatState.busy) return;
  chatState.error = null;

  const userMsg: ChatMessage = {
    id: newId(),
    role: "user",
    content: text,
  };
  const assistantMsg: ChatMessage = {
    id: newId(),
    role: "assistant",
    content: "",
    streaming: true,
    contextMode: chatState.contextMode,
  };
  chatState.messages = [...chatState.messages, userMsg, assistantMsg];
  chatState.busy = true;

  const apiMessages = chatState.messages
    .filter((m) => m.role === "user" || (m.role === "assistant" && m.content))
    .slice(0, -1) // exclude the just-added empty assistant placeholder
    .map((m) => ({ role: m.role, content: m.content }));

  try {
    const turnId = await invoke<string>("chat_send", {
      messages: apiMessages,
      model: chatState.model,
      contextMode: chatState.contextMode,
      currentRelPath,
    });
    chatState.activeTurnId = turnId;
  } catch (e) {
    assistantMsg.streaming = false;
    chatState.busy = false;
    chatState.error = String(e);
  }
}

export function clearConversation() {
  chatState.messages = [];
  chatState.error = null;
  chatState.activeTurnId = null;
  chatState.busy = false;
}
