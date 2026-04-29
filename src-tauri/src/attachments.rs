// Phase 11 — paste/drop image attachments.
//
// Attachments live BESIDE the page they were first inserted into:
//   - For a top-level page (Foo.md), the file lands at the vault root.
//   - For a page in a book (Daily/2026-04-21.md), it lands inside the book
//     folder (Daily/).
//
// File names are content-addressed (12-hex-char SHA-256 prefix) so the
// same paste in two places dedupes within a folder. Extension is sanitized
// to alphanumerics so we can't write executables or weird suffixes.

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::vault::Vault;

const HASH_PREFIX_LEN: usize = 12;

pub fn save_attachment_from_path(
    vault: &Vault,
    page_rel_path: &str,
    src_path: &str,
) -> Result<String> {
    let p = Path::new(src_path);
    let bytes = std::fs::read(p).with_context(|| format!("reading {}", p.display()))?;
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    save_attachment(vault, page_rel_path, ext, &bytes)
}

pub fn save_attachment(
    vault: &Vault,
    page_rel_path: &str,
    ext: &str,
    bytes: &[u8],
) -> Result<String> {
    if bytes.is_empty() {
        return Err(anyhow!("empty attachment"));
    }

    // Resolve the folder for the active page.
    let page_full = vault.root.join(page_rel_path);
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

    let safe_ext: String = ext
        .trim_start_matches('.')
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(8)
        .collect();
    let safe_ext = if safe_ext.is_empty() {
        "bin".to_string()
    } else {
        safe_ext
    };

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
