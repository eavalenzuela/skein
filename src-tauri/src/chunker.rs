// Section-based markdown chunker for embeddings.
//
// Splits on ATX headings (`#`, `##`, `###`); each section becomes one chunk.
// Sections without a heading get a single "preface" chunk. Long sections are
// further split at blank-line boundaries with a soft cap.

#[derive(Debug, Clone)]
pub struct Chunk {
    pub idx: u32,
    pub heading: String,
    pub text: String,
}

const SOFT_CAP_CHARS: usize = 2000; // ~500 tokens at ~4 chars/token

pub fn chunk_markdown(body: &str) -> Vec<Chunk> {
    let body = strip_frontmatter(body);
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut current_heading = String::new();
    let mut current_text = String::new();
    let mut idx: u32 = 0;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = strip_atx(trimmed) {
            // Flush whatever we accumulated under the previous heading.
            flush(&mut chunks, &mut idx, &current_heading, &mut current_text);
            current_heading = rest.trim().to_string();
            current_text.clear();
        } else {
            current_text.push_str(line);
            current_text.push('\n');
            if current_text.len() >= SOFT_CAP_CHARS {
                // Try to break at the last blank line for readability; fall
                // back to the soft cap if there's no blank line in range.
                if let Some(cut) = current_text[..SOFT_CAP_CHARS].rfind("\n\n") {
                    let head: String = current_text[..cut + 1].to_string();
                    let tail: String = current_text[cut + 2..].to_string();
                    push_chunk(&mut chunks, &mut idx, &current_heading, head);
                    current_text = tail;
                } else {
                    let head = std::mem::take(&mut current_text);
                    push_chunk(&mut chunks, &mut idx, &current_heading, head);
                }
            }
        }
    }

    flush(&mut chunks, &mut idx, &current_heading, &mut current_text);
    chunks
}

fn strip_atx(line: &str) -> Option<&str> {
    let l = line;
    let count = l.bytes().take_while(|&b| b == b'#').count();
    if count == 0 || count > 6 {
        return None;
    }
    let rest = &l[count..];
    if rest.starts_with(' ') || rest.is_empty() {
        Some(rest.trim_start_matches(' '))
    } else {
        None
    }
}

fn strip_frontmatter(body: &str) -> &str {
    if !body.starts_with("---\n") {
        return body;
    }
    let after_open = &body[4..];
    if let Some(end_rel) = after_open.find("\n---\n") {
        return &after_open[end_rel + 5..];
    }
    if let Some(end_rel) = after_open.find("\n---") {
        return &after_open[end_rel + 4..];
    }
    body
}

fn push_chunk(chunks: &mut Vec<Chunk>, idx: &mut u32, heading: &str, text: String) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    chunks.push(Chunk {
        idx: *idx,
        heading: heading.to_string(),
        text: trimmed.to_string(),
    });
    *idx += 1;
}

fn flush(chunks: &mut Vec<Chunk>, idx: &mut u32, heading: &str, text: &mut String) {
    if !text.trim().is_empty() {
        push_chunk(chunks, idx, heading, std::mem::take(text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_by_heading() {
        let md = "Preface text.\n\n# H1\nUnder one.\n\n## H2\nUnder two.";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].heading, "");
        assert_eq!(chunks[1].heading, "H1");
        assert_eq!(chunks[2].heading, "H2");
    }

    #[test]
    fn strips_frontmatter() {
        let md = "---\ntitle: x\n---\n\n# Hello\nbody";
        let chunks = chunk_markdown(md);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading, "Hello");
    }
}
