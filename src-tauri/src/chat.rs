// Phase 7 — Anthropic chat with prompt caching, RAG, and SSE streaming.
//
// The chat lives entirely server-side: the JS sends a list of conversation
// messages plus a context-mode + active-page hint, and we shape the request
// (system prompt + retrieved context block + cache_control), call the
// Anthropic API with stream=true, parse SSE, and emit per-token events back
// to the frontend on the "chat-event" channel.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Runtime};

use crate::index::Index;
use crate::vault::Vault;

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessageIn {
    pub role: String,    // "user" | "assistant"
    pub content: String, // plain text only for v1
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatEvent {
    pub turn_id: String,
    pub kind: String, // "started" | "token" | "done" | "error"
    pub text: Option<String>,
    pub error: Option<String>,
    /// For "done" — describe the context that was sent so the UI can show it.
    pub context: Option<ChatContextInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatContextInfo {
    pub mode: String,
    pub current_rel_path: Option<String>,
    /// Chunks pulled into the system prompt as context.
    pub chunks: Vec<ContextChunk>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextChunk {
    pub rel_path: String,
    pub title: String,
    pub heading: String,
    pub similarity: f32,
}

const ANTHROPIC_API: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

const SYSTEM_PROMPT: &str = "\
You are Skein, a writing companion built into a local note-taking app. \
The user is the owner of the vault. Be concise, literary in tone, and \
respect the user's existing voice. When you cite something from the vault, \
quote it accurately and name the source file in backticks. If you don't \
know something, say so plainly rather than inventing.";

#[derive(Debug, Clone)]
pub struct PreparedRequest {
    pub body: Value,
    pub context: ChatContextInfo,
}

/// Build the Anthropic request body and the corresponding context info.
pub fn prepare_request(
    messages: &[ChatMessageIn],
    model: &str,
    max_tokens: u32,
    context_mode: &str,
    current_rel_path: Option<&str>,
    vault: Option<&Vault>,
    index: &Mutex<Option<Index>>,
) -> Result<PreparedRequest> {
    let mut chunks: Vec<ContextChunk> = Vec::new();
    let mut context_block = String::new();

    // The thing we'll embed to drive RAG retrieval. Default: the latest
    // user message; for whole-vault we still RAG against it because we
    // can't fit a full vault in.
    let query_text = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let want_current = matches!(context_mode, "current" | "current+related" | "vault");
    let want_related = matches!(context_mode, "current+related" | "vault");

    // Current note body, if asked for.
    if want_current {
        if let (Some(v), Some(rp)) = (vault, current_rel_path) {
            if let Ok(body) = crate::vault::read_page_body(v, rp) {
                context_block.push_str(&format!(
                    "## Current note: `{}`\n\n{}\n\n",
                    rp,
                    body.trim()
                ));
            }
        }
    }

    // Top-K related chunks.
    if want_related && !query_text.trim().is_empty() {
        let k = if context_mode == "vault" { 16 } else { 8 };
        if let Some(idx) = index.lock().as_ref() {
            if let Ok(hits) = idx.retrieve_chunks(&query_text, k, current_rel_path) {
                if !hits.is_empty() {
                    context_block.push_str("## Related chunks from the vault\n\n");
                    for hit in &hits {
                        let label = if hit.heading.is_empty() {
                            hit.title.clone()
                        } else {
                            format!("{} — {}", hit.title, hit.heading)
                        };
                        context_block.push_str(&format!(
                            "### `{}` ({})\n\n{}\n\n",
                            hit.rel_path,
                            label,
                            hit.text.trim()
                        ));
                        chunks.push(ContextChunk {
                            rel_path: hit.rel_path.clone(),
                            title: hit.title.clone(),
                            heading: hit.heading.clone(),
                            similarity: hit.similarity,
                        });
                    }
                }
            }
        }
    }

    // Build the system prompt as multi-block so we can put cache_control on
    // the heavy retrieved-context block.
    let system: Value = if context_block.is_empty() {
        json!([{ "type": "text", "text": SYSTEM_PROMPT }])
    } else {
        json!([
            { "type": "text", "text": SYSTEM_PROMPT },
            {
                "type": "text",
                "text": context_block,
                "cache_control": { "type": "ephemeral" }
            }
        ])
    };

    let api_messages: Vec<Value> = messages
        .iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| json!({ "role": m.role, "content": m.content }))
        .collect();

    let body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "stream": true,
        "system": system,
        "messages": api_messages,
    });

    Ok(PreparedRequest {
        body,
        context: ChatContextInfo {
            mode: context_mode.to_string(),
            current_rel_path: current_rel_path.map(|s| s.to_string()),
            chunks,
        },
    })
}

pub async fn run_chat<R: Runtime>(
    app: AppHandle<R>,
    turn_id: String,
    api_key: String,
    body: Value,
    context: ChatContextInfo,
) {
    if let Err(e) = run_chat_inner(&app, &turn_id, &api_key, body, context.clone()).await {
        let _ = app.emit(
            "chat-event",
            ChatEvent {
                turn_id: turn_id.clone(),
                kind: "error".into(),
                text: None,
                error: Some(e.to_string()),
                context: None,
            },
        );
    }
}

async fn run_chat_inner<R: Runtime>(
    app: &AppHandle<R>,
    turn_id: &str,
    api_key: &str,
    body: Value,
    context: ChatContextInfo,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let _ = app.emit(
        "chat-event",
        ChatEvent {
            turn_id: turn_id.to_string(),
            kind: "started".into(),
            text: None,
            error: None,
            context: None,
        },
    );

    let res = client
        .post(ANTHROPIC_API)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .context("calling Anthropic API")?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(anyhow!("anthropic API {}: {}", status, text));
    }

    let mut stream = res.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        // Each SSE event ends with a blank line.
        while let Some(pos) = buffer.find("\n\n") {
            let event_block: String = buffer.drain(..pos + 2).collect();
            let trimmed = event_block.trim_end_matches('\n');
            let mut event_type = String::new();
            let mut data = String::new();
            for line in trimmed.lines() {
                if let Some(rest) = line.strip_prefix("event: ") {
                    event_type = rest.to_string();
                } else if let Some(rest) = line.strip_prefix("data: ") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(rest);
                }
            }
            if event_type == "content_block_delta" {
                if let Ok(v) = serde_json::from_str::<Value>(&data) {
                    if let Some(text) = v
                        .get("delta")
                        .and_then(|d| d.get("text"))
                        .and_then(|t| t.as_str())
                    {
                        let _ = app.emit(
                            "chat-event",
                            ChatEvent {
                                turn_id: turn_id.to_string(),
                                kind: "token".into(),
                                text: Some(text.to_string()),
                                error: None,
                                context: None,
                            },
                        );
                    }
                }
            }
        }
    }

    let _ = app.emit(
        "chat-event",
        ChatEvent {
            turn_id: turn_id.to_string(),
            kind: "done".into(),
            text: None,
            error: None,
            context: Some(context),
        },
    );
    Ok(())
}

/// Drives a chat turn. Spawns the streaming task and returns the turn id
/// so the caller can correlate "chat-event" events.
pub fn spawn_chat<R: Runtime + 'static>(
    app: AppHandle<R>,
    api_key: String,
    body: Value,
    context: ChatContextInfo,
) -> String {
    let turn_id = uuid::Uuid::new_v4().to_string();
    let app_for_task = app.clone();
    let id_for_task = turn_id.clone();
    tauri::async_runtime::spawn(async move {
        run_chat(app_for_task, id_for_task, api_key, body, context).await;
    });
    turn_id
}
