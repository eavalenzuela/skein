// End-to-end tests for the Skein vault layer against real tempdir
// vaults. These exercise the same `books`, `pages`, and `vault` modules
// the Tauri commands call into, but skip the IPC layer so they run as
// plain `cargo test`.

use std::fs;

use skein_lib::*;
use tempfile::TempDir;

fn fresh_vault() -> (TempDir, vault::Vault) {
    let dir = TempDir::new().expect("tempdir");
    // Make sure the vault root resolves through canonicalize() even on
    // macOS where /tmp is symlinked to /private/tmp.
    let v = vault::Vault::from_path(dir.path().to_path_buf()).expect("vault");
    (dir, v)
}

fn read_to_string(path: impl AsRef<std::path::Path>) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn create_book_makes_a_folder() {
    let (_tmp, v) = fresh_vault();
    books::create_book(&v, "Research").unwrap();
    assert!(v.root.join("Research").is_dir());
}

#[test]
fn create_book_rejects_dot_prefix_and_path_separators() {
    let (_tmp, v) = fresh_vault();
    assert!(books::create_book(&v, ".secret").is_err());
    assert!(books::create_book(&v, "../escape").is_err());
    assert!(books::create_book(&v, "with/slash").is_err());
    assert!(books::create_book(&v, "").is_err());
}

#[test]
fn create_book_rejects_duplicate() {
    let (_tmp, v) = fresh_vault();
    books::create_book(&v, "Daily").unwrap();
    let err = books::create_book(&v, "Daily").unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn rename_book_updates_meta_order() {
    let (_tmp, v) = fresh_vault();
    books::create_book(&v, "A").unwrap();
    books::create_book(&v, "B").unwrap();
    books::write_meta(
        &v,
        &books::BooksMeta {
            order: vec!["A".into(), "B".into()],
        },
    )
    .unwrap();
    books::rename_book(&v, "A", "Alpha").unwrap();
    let meta = books::read_meta(&v);
    assert_eq!(meta.order, vec!["Alpha".to_string(), "B".to_string()]);
    assert!(v.root.join("Alpha").is_dir());
    assert!(!v.root.join("A").exists());
}

#[test]
fn delete_book_with_pages_moves_them_to_folio() {
    let (_tmp, v) = fresh_vault();
    books::create_book(&v, "Trash").unwrap();
    pages::create_page(&v, Some("Trash"), "kept").unwrap();
    let r = books::delete_book(&v, "Trash", false).unwrap();
    assert_eq!(r.moved.len(), 1);
    assert!(v.root.join("kept.md").is_file());
    assert!(!v.root.join("Trash").exists());
}

#[test]
fn delete_book_also_delete_pages_drops_files() {
    let (_tmp, v) = fresh_vault();
    books::create_book(&v, "Gone").unwrap();
    pages::create_page(&v, Some("Gone"), "doomed").unwrap();
    let r = books::delete_book(&v, "Gone", true).unwrap();
    assert_eq!(r.deleted_rel_paths.len(), 1);
    assert!(!v.root.join("Gone").exists());
}

#[test]
fn create_page_writes_frontmatter_and_returns_rel_path() {
    let (_tmp, v) = fresh_vault();
    let rel = pages::create_page(&v, None, "First note").unwrap();
    assert_eq!(rel, "First note.md");
    let body = read_to_string(v.root.join(&rel));
    assert!(body.starts_with("---\ntitle: First note\n---\n"));
}

#[test]
fn create_page_in_book_lands_in_book_folder() {
    let (_tmp, v) = fresh_vault();
    books::create_book(&v, "Recipes").unwrap();
    let rel = pages::create_page(&v, Some("Recipes"), "cake").unwrap();
    assert_eq!(rel, "Recipes/cake.md");
    assert!(v.root.join("Recipes/cake.md").is_file());
}

#[test]
fn create_page_disambiguates_collision_with_numeric_suffix() {
    let (_tmp, v) = fresh_vault();
    pages::create_page(&v, None, "draft").unwrap();
    let rel2 = pages::create_page(&v, None, "draft").unwrap();
    assert_eq!(rel2, "draft (1).md");
}

#[test]
fn create_page_sanitizes_path_chars() {
    let (_tmp, v) = fresh_vault();
    let rel = pages::create_page(&v, None, "weird/title?stars*").unwrap();
    // All forbidden chars are replaced with `-`.
    assert!(rel.contains("weird-title-stars-"));
    assert!(v.root.join(&rel).is_file());
}

#[test]
fn rename_page_rewrites_wikilinks_in_other_pages() {
    let (_tmp, v) = fresh_vault();
    pages::create_page(&v, None, "Old Page").unwrap();
    pages::create_page(&v, None, "Linker").unwrap();
    fs::write(
        v.root.join("Linker.md"),
        "---\ntitle: Linker\n---\n\nsee [[Old Page]] and [[Old Page|alias]]\n",
    )
    .unwrap();
    pages::rename_page(&v, "Old Page.md", "New Page").unwrap();
    let after = read_to_string(v.root.join("Linker.md"));
    assert!(after.contains("[[New Page]]"));
    assert!(after.contains("[[New Page|alias]]"));
    assert!(!after.contains("[[Old Page"));
}

#[test]
fn delete_page_removes_file() {
    let (_tmp, v) = fresh_vault();
    let rel = pages::create_page(&v, None, "doomed").unwrap();
    pages::delete_page(&v, &rel).unwrap();
    assert!(!v.root.join(&rel).exists());
}

#[test]
fn list_books_honors_explicit_order() {
    let (_tmp, v) = fresh_vault();
    books::create_book(&v, "Alpha").unwrap();
    books::create_book(&v, "Beta").unwrap();
    books::create_book(&v, "Gamma").unwrap();
    books::write_meta(
        &v,
        &books::BooksMeta {
            order: vec!["Gamma".into(), "Alpha".into()],
        },
    )
    .unwrap();
    let listed = vault::list_books(&v).unwrap();
    let ordered = books::apply_order(listed, &books::read_meta(&v));
    let names: Vec<_> = ordered.iter().map(|b| b.name.as_str()).collect();
    // Gamma and Alpha first in their explicit order, then Beta as the
    // alphabetically-sorted leftover.
    assert_eq!(names, vec!["Gamma", "Alpha", "Beta"]);
}

#[test]
fn list_loose_pages_excludes_dot_files_and_book_pages() {
    let (_tmp, v) = fresh_vault();
    pages::create_page(&v, None, "loose").unwrap();
    books::create_book(&v, "Book").unwrap();
    pages::create_page(&v, Some("Book"), "in-book").unwrap();
    fs::write(v.root.join(".hidden.md"), "---\ntitle: hidden\n---\n").unwrap();
    let loose = vault::list_loose_pages(&v).unwrap();
    let titles: Vec<_> = loose.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, vec!["loose"]);
}

#[test]
fn read_then_write_page_round_trips_body() {
    let (_tmp, v) = fresh_vault();
    let rel = pages::create_page(&v, None, "round").unwrap();
    let edited = "---\ntitle: round\n---\n\nbody after edit\n";
    vault::write_page_body(&v, &rel, edited).unwrap();
    assert_eq!(vault::read_page_body(&v, &rel).unwrap(), edited);
}

#[test]
fn write_page_rejects_path_escape() {
    let (_tmp, v) = fresh_vault();
    // Caller can't smuggle a path that escapes the vault.
    let err = vault::write_page_body(&v, "../escape.md", "x").unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("escape")
            || err.to_string().to_lowercase().contains("not found")
            || err.to_string().to_lowercase().contains("no such")
    );
}
