// Phase 11 — paste/drop image attachments.
//
// Attachments live BESIDE the page they were first inserted into:
//   - For a top-level page (Foo.md), the file lands at the vault root.
//   - For a page in a book (Daily/2026-04-21.md), it lands inside the book
//     folder (Daily/).
//
// File names are content-addressed (12-hex-char SHA-256 prefix) so the
// same paste in two places dedupes within a folder. The extension comes
// from sniffing the bytes, not from the caller — the vault is a git working
// tree and a zip others may open, so only real images get written.

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::vault::{resolve_in_vault, Vault};

const HASH_PREFIX_LEN: usize = 12;
/// 64 MiB. Well past any real screenshot, short of exhausting memory on a
/// mistaken drop of a video file.
const MAX_ATTACHMENT_BYTES: usize = 64 * 1024 * 1024;

/// Identify an image from its leading bytes. Returns the canonical
/// extension we will store it under, or None when it isn't an image we
/// recognize — the only gate on what enters the vault.
fn sniff_image_ext(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("png");
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("jpg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("gif");
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("webp");
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        let brand = &bytes[8..12];
        if brand == b"avif" || brand == b"avis" {
            return Some("avif");
        }
    }
    if bytes.starts_with(b"BM") {
        return Some("bmp");
    }
    None
}

pub fn save_attachment_from_path(
    vault: &Vault,
    page_rel_path: &str,
    src_path: &str,
) -> Result<String> {
    let p = Path::new(src_path);
    // Cap the read: the source is an arbitrary path from the file dialog and
    // a huge file would otherwise be slurped whole before any check.
    let meta = std::fs::metadata(p).with_context(|| format!("reading {}", p.display()))?;
    if meta.len() as usize > MAX_ATTACHMENT_BYTES {
        return Err(anyhow!("attachment is larger than 64 MB"));
    }
    let bytes = std::fs::read(p).with_context(|| format!("reading {}", p.display()))?;
    save_attachment(vault, page_rel_path, "", &bytes)
}

/// `ext` is advisory only — kept for call-site clarity — and is ignored in
/// favour of what the bytes actually are.
pub fn save_attachment(
    vault: &Vault,
    page_rel_path: &str,
    _ext: &str,
    bytes: &[u8],
) -> Result<String> {
    if bytes.is_empty() {
        return Err(anyhow!("empty attachment"));
    }
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Err(anyhow!("attachment is larger than 64 MB"));
    }
    let safe_ext = sniff_image_ext(bytes)
        .ok_or_else(|| anyhow!("attachment is not a supported image (png/jpg/gif/webp/avif/bmp)"))?;

    // Resolve the folder for the active page. Validate before creating so a
    // rejected path leaves no directories behind.
    let page_full = resolve_in_vault(vault, page_rel_path, false)?;
    let folder = page_full
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| vault.root.clone());
    std::fs::create_dir_all(&folder)?;
    let canonical_folder = folder
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", folder.display()))?;
    if !canonical_folder.starts_with(&vault.root) {
        return Err(anyhow!("path escapes vault root"));
    }

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let full_hash = format!("{:x}", hasher.finalize());
    let prefix = &full_hash[..HASH_PREFIX_LEN];

    let filename = format!("{prefix}.{safe_ext}");
    let target = canonical_folder.join(&filename);

    // Skip writing if a same-content file already exists (content-addressed
    // dedup; no point rewriting the same bytes).
    if !target.exists() {
        std::fs::write(&target, bytes).with_context(|| format!("writing {}", target.display()))?;
    }

    let rel = target
        .strip_prefix(&vault.root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.replace('\\', "/"))
        .ok_or_else(|| anyhow!("non-utf8 attachment path"))?;
    Ok(rel)
}
