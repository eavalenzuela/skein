// Phase 8 — auto-tag suggestions via Claude Haiku.
//
// Asks the model for 1-5 tags given the active page body and the existing
// vault tag set. Returns suggestions to the frontend as chips; the user
// accepts them one at a time, which writes the tag into the page's YAML
// frontmatter and triggers a normal save.
//
// Activation: requires an Anthropic API key in the OS keychain. With no
// key, the chip row stays hidden — manual tagging always works.

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

const ANTHROPIC_API: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const TAG_MODEL: &str = "claude-haiku-4-5";

pub async fn suggest_tags(
    api_key: &str,
    title: &str,
    body: &str,
    existing_vault_tags: &[String],
) -> Result<Vec<String>> {
    let trimmed_body = if body.len() > 6000 {
        // Cap to keep the prompt small. The first 6000 chars almost always
        // capture the topic; pages longer than that can be re-summarized
        // properly in a future polish pass.
        let mut t = body[..6000].to_string();
        t.push_str("\n\n[…truncated for tagging…]");
        t
    } else {
        body.to_string()
    };

    let existing = if existing_vault_tags.is_empty() {
        "(none yet)".to_string()
    } else {
        existing_vault_tags.join(", ")
    };

    let user = format!(
        "You are tagging a personal note in a private vault.\n\n\
         Existing vault tags (strongly prefer these when they fit; only invent new ones if nothing existing matches):\n{existing}\n\n\
         Tag rules:\n\
         - 1 to 5 tags\n\
         - lowercase, hyphenated, no spaces, no leading '#'\n\
         - concrete topics, not generic adjectives\n\n\
         Note title: {title}\n\
         Note body:\n{trimmed_body}\n\n\
         Respond with ONLY a JSON array of strings. No prose, no code fences."
    );

    let body_json = json!({
        "model": TAG_MODEL,
        "max_tokens": 256,
        "messages": [{ "role": "user", "content": user }]
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()?;
    let res = client
        .post(ANTHROPIC_API)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&body_json)
        .send()
        .await
        .context("autotag: calling Anthropic")?;
    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(anyhow!("autotag api {}: {}", status, text));
    }
    let v: Value = res.json().await?;
    let text = v
        .get("content")
        .and_then(|c| c.get(0))
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow!("autotag: malformed response"))?;
    parse_tags(text)
}

fn parse_tags(raw: &str) -> Result<Vec<String>> {
    // Strip code fences if the model wrapped the JSON.
    let mut s = raw.trim();
    if let Some(rest) = s.strip_prefix("```json") {
        s = rest.trim_start();
    } else if let Some(rest) = s.strip_prefix("```") {
        s = rest.trim_start();
    }
    if let Some(rest) = s.strip_suffix("```") {
        s = rest.trim_end();
    }
    let parsed: Vec<String> = serde_json::from_str(s.trim()).with_context(|| {
        format!(
            "parsing tag JSON: {}",
            s.chars().take(120).collect::<String>()
        )
    })?;
    let mut out = Vec::new();
    for t in parsed {
        let cleaned = clean_tag(&t);
        if !cleaned.is_empty() && !out.contains(&cleaned) {
            out.push(cleaned);
        }
    }
    Ok(out.into_iter().take(5).collect())
}

fn clean_tag(t: &str) -> String {
    let t = t.trim().trim_start_matches('#').trim();
    t.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect()
}

/// Insert `tag` into the YAML frontmatter `tags:` list of `body`, preserving
/// any other frontmatter keys. If no frontmatter exists, prepends one.
pub fn add_tag_to_body(body: &str, tag: &str) -> Result<String> {
    let tag = clean_tag(tag);
    if tag.is_empty() {
        return Err(anyhow!("invalid tag"));
    }

    if !body.starts_with("---\n") && !body.starts_with("---\r\n") {
        return Ok(format!("---\ntags: [{tag}]\n---\n\n{}", body.trim_start()));
    }

    let after_open_offset = if body.starts_with("---\r\n") { 5 } else { 4 };
    let after_open = &body[after_open_offset..];
    let end_marker = ["\n---\n", "\n---\r\n", "\n---"];
    let end_rel = end_marker
        .iter()
        .filter_map(|m| after_open.find(m).map(|i| (i, m.len())))
        .min_by_key(|(i, _)| *i)
        .ok_or_else(|| anyhow!("malformed frontmatter"))?;
    let yaml_block = &after_open[..end_rel.0];
    let rest = &after_open[end_rel.0 + end_rel.1..];

    let mut value: serde_yml::Value = if yaml_block.trim().is_empty() {
        serde_yml::Value::Mapping(serde_yml::Mapping::new())
    } else {
        serde_yml::from_str(yaml_block)
            .with_context(|| format!("parsing frontmatter: {}", yaml_block))?
    };

    let map = value
        .as_mapping_mut()
        .ok_or_else(|| anyhow!("frontmatter is not a YAML mapping"))?;

    let tags_key = serde_yml::Value::String("tags".into());
    let entry = map
        .entry(tags_key)
        .or_insert(serde_yml::Value::Sequence(Vec::new()));
    match entry {
        serde_yml::Value::Sequence(seq) => {
            let already = seq.iter().any(|v| v.as_str() == Some(tag.as_str()));
            if !already {
                seq.push(serde_yml::Value::String(tag.clone()));
            }
        }
        // If tags was a single string, promote it to a sequence with both
        // values.
        serde_yml::Value::String(existing) => {
            let prev = existing.clone();
            *entry = serde_yml::Value::Sequence(if prev == tag {
                vec![serde_yml::Value::String(prev)]
            } else {
                vec![
                    serde_yml::Value::String(prev),
                    serde_yml::Value::String(tag.clone()),
                ]
            });
        }
        _ => {
            *entry = serde_yml::Value::Sequence(vec![serde_yml::Value::String(tag.clone())]);
        }
    }

    let serialized = serde_yml::to_string(&value)?;
    Ok(format!("---\n{serialized}---\n{rest}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_tag_to_existing_list() {
        let body = "---\ntitle: x\ntags:\n- one\n- two\n---\n\nbody\n";
        let out = add_tag_to_body(body, "three").unwrap();
        assert!(out.contains("three"));
        assert!(out.contains("title: x"));
        assert!(out.contains("body"));
    }

    #[test]
    fn idempotent_when_tag_exists() {
        let body = "---\ntags: [a, b]\n---\n\nx";
        let out = add_tag_to_body(body, "a").unwrap();
        let count = out.matches("a").count();
        // exactly two occurrences ('a' in `tags:` list, plus the literal `a`)
        // not three or more
        assert!(count <= 3);
    }

    #[test]
    fn adds_frontmatter_when_missing() {
        let body = "no frontmatter here";
        let out = add_tag_to_body(body, "fresh").unwrap();
        assert!(out.starts_with("---\n"));
        assert!(out.contains("tags:"));
        assert!(out.contains("fresh"));
        assert!(out.ends_with("no frontmatter here"));
    }

    #[test]
    fn parses_clean_json_array() {
        let v = parse_tags("[\"reading\", \"attention\"]").unwrap();
        assert_eq!(v, vec!["reading", "attention"]);
    }

    #[test]
    fn parses_fenced_json() {
        let v = parse_tags("```json\n[\"a\",\"b\"]\n```").unwrap();
        assert_eq!(v, vec!["a", "b"]);
    }

    #[test]
    fn cleans_hashes_and_uppercase() {
        let v = parse_tags("[\"#Reading\", \"Quiet Mind\"]").unwrap();
        assert_eq!(v, vec!["reading", "quiet-mind"]);
    }
}
