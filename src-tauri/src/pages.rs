// Phase E — page CRUD.
//
// Create a new markdown page in the vault root or in a book folder, rename
// an existing page (with vault-wide wikilink rewrites so [[Old]] becomes
// [[New]]), and delete a page from disk.
//
// Wikilinks resolve by filename stem. Renaming a page rewrites wikilinks in
// every other page that targets the same stem — this can over-match in the
// rare case that two different books contain pages with identical stems,
// but it's the correct behavior in the overwhelming majority of cases and
// avoids leaving dangling links after a rename.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use walkdir::WalkDir;

use crate::vault::Vault;

const FORBIDDEN_STEMS: &[&str] = &[".", "..", ""];

pub fn create_page(vault: &Vault, book: Option<&str>, title: &str) -> Result<String> {
    let stem = sanitize_stem(title)?;
    let dir = match book {
        Some(b) => vault.root.join(b),
        None => vault.root.clone(),
    };
    if !dir.is_dir() {
        return Err(anyhow!("destination folder not found"));
    }
    let canonical_dir = dir.canonicalize()?;
    if !canonical_dir.starts_with(&vault.root) {
        return Err(anyhow!("path escapes vault root"));
    }
    let final_stem = unique_stem(&canonical_dir, &stem);
    let target = canonical_dir.join(format!("{final_stem}.md"));
    let body = format!("---\ntitle: {final_stem}\n---\n\n");
    fs::write(&target, body)
        .with_context(|| format!("writing {}", target.display()))?;
    rel_string(&vault.root, &target)
}

pub fn rename_page(vault: &Vault, old_rel: &str, new_title: &str) -> Result<String> {
    let new_stem = sanitize_stem(new_title)?;
    let old_full = vault.root.join(old_rel);
    if !old_full.is_file() {
        return Err(anyhow!("page not found: {old_rel}"));
    }
    let parent = old_full
        .parent()
        .ok_or_else(|| anyhow!("page has no parent dir"))?
        .to_path_buf();
    let old_stem = old_full
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("page filename is not utf-8"))?
        .to_string();
    if new_stem == old_stem {
        return Ok(old_rel.to_string());
    }
    let final_stem = unique_stem(&parent, &new_stem);
    let new_full = parent.join(format!("{final_stem}.md"));
    fs::rename(&old_full, &new_full)
        .with_context(|| format!("renaming {} → {}", old_full.display(), new_full.display()))?;

    rewrite_wikilinks(vault, &old_stem, &final_stem)?;

    rel_string(&vault.root, &new_full)
}

pub fn delete_page(vault: &Vault, rel: &str) -> Result<()> {
    let full = vault.root.join(rel);
    if !full.is_file() {
        return Err(anyhow!("page not found: {rel}"));
    }
    let canonical = full.canonicalize()?;
    if !canonical.starts_with(&vault.root) {
        return Err(anyhow!("path escapes vault root"));
    }
    fs::remove_file(&canonical)
        .with_context(|| format!("removing {}", canonical.display()))?;
    Ok(())
}

fn sanitize_stem(title: &str) -> Result<String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("title is empty"));
    }
    if trimmed.len() > 100 {
        return Err(anyhow!("title is too long"));
    }
    let cleaned: String = trimmed
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' | '\n' | '\r' | '\t' => '-',
            other => other,
        })
        .collect();
    let cleaned = cleaned.trim().to_string();
    if FORBIDDEN_STEMS.contains(&cleaned.as_str()) {
        return Err(anyhow!("title is reserved"));
    }
    if cleaned.starts_with('.') {
        return Err(anyhow!("title cannot start with `.`"));
    }
    Ok(cleaned)
}

fn unique_stem(dir: &Path, stem: &str) -> String {
    if !dir.join(format!("{stem}.md")).exists() {
        return stem.to_string();
    }
    for n in 1.. {
        let candidate = format!("{stem} ({n})");
        if !dir.join(format!("{candidate}.md")).exists() {
            return candidate;
        }
    }
    unreachable!()
}

fn rel_string(root: &Path, full: &Path) -> Result<String> {
    full.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.replace('\\', "/"))
        .ok_or_else(|| anyhow!("non-utf-8 path"))
}

fn rewrite_wikilinks(vault: &Vault, old_stem: &str, new_stem: &str) -> Result<()> {
    let mut targets: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&vault.root)
        .max_depth(2)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let Some(ext) = p.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("md") && !ext.eq_ignore_ascii_case("markdown") {
            continue;
        }
        // Skip files in reserved dot-folders (e.g. `.skein/`). Test only
        // path components below the vault root so a vault that happens to
        // live inside a dot-folder (e.g. `/home/me/.notes`) still works.
        let in_dot_folder = p
            .strip_prefix(&vault.root)
            .ok()
            .map(|rel| {
                rel.components()
                    .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
            })
            .unwrap_or(false);
        if in_dot_folder {
            continue;
        }
        targets.push(p.to_path_buf());
    }
    for path in targets {
        let body = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let rewritten = rewrite_wikilinks_in_body(&body, old_stem, new_stem);
        if rewritten != body {
            fs::write(&path, rewritten)
                .with_context(|| format!("rewriting {}", path.display()))?;
        }
    }
    Ok(())
}

/// Replace `[[old]]` and `[[old|alias]]` with `[[new]]` (preserving alias)
/// throughout `body`. Targets are matched case-sensitively against the full
/// inner text before any pipe — same convention the indexer uses.
fn rewrite_wikilinks_in_body(body: &str, old: &str, new: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(idx) = rest.find("[[") {
        out.push_str(&rest[..idx]);
        let after = &rest[idx + 2..];
        let Some(end) = after.find("]]") else {
            // Unterminated `[[` — copy the rest verbatim and bail.
            out.push_str("[[");
            out.push_str(after);
            return out;
        };
        let inner = &after[..end];
        if inner.contains('\n') {
            // A linebreak inside `[[...]]` is malformed; copy as-is.
            out.push_str("[[");
            out.push_str(inner);
            out.push_str("]]");
            rest = &after[end + 2..];
            continue;
        }
        let (target, alias) = match inner.split_once('|') {
            Some((t, a)) => (t.trim(), Some(a)),
            None => (inner.trim(), None),
        };
        if target == old {
            out.push_str("[[");
            out.push_str(new);
            if let Some(a) = alias {
                out.push('|');
                out.push_str(a);
            }
            out.push_str("]]");
        } else {
            out.push_str("[[");
            out.push_str(inner);
            out.push_str("]]");
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_plain_wikilink() {
        let body = "see [[Old Page]] for more.";
        assert_eq!(
            rewrite_wikilinks_in_body(body, "Old Page", "New Page"),
            "see [[New Page]] for more."
        );
    }

    #[test]
    fn rewrites_aliased_wikilink_preserving_alias() {
        let body = "ref [[Old|the old one]] here";
        assert_eq!(
            rewrite_wikilinks_in_body(body, "Old", "New"),
            "ref [[New|the old one]] here"
        );
    }

    #[test]
    fn leaves_other_targets_alone() {
        let body = "[[Old]] vs [[Other]]";
        assert_eq!(
            rewrite_wikilinks_in_body(body, "Old", "New"),
            "[[New]] vs [[Other]]"
        );
    }

    #[test]
    fn unterminated_brackets_left_alone() {
        // The first `[[` swallows everything up to the next `]]`, so the
        // legit `[[Old]]` later in the line stays unchanged. Garbage in
        // garbage out — same as the indexer's parser.
        let body = "broken [[ no close on this line";
        assert_eq!(
            rewrite_wikilinks_in_body(body, "Old", "New"),
            "broken [[ no close on this line"
        );
    }
}
