// FTS-only tests for the index layer. The embedder is left in
// hash-bag fallback mode (no fastembed model download) since these
// tests only assert search/backlinks behavior, not semantic similarity.

use std::sync::Arc;

use skein_lib::*;
use tempfile::TempDir;

fn fresh_index() -> (TempDir, index::Index) {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("index.db");
    // Hash-bag embedder = no model download. FTS search still works
    // normally — only semantic similarity uses the embedder.
    let embedder: embedder::SharedEmbedder = Arc::new(embedder::HashBagEmbedder::new());
    let idx = index::Index::open(&db, embedder).expect("open index");
    (dir, idx)
}

fn page(rel: &str, title: &str, body: &str, book: Option<&str>) -> vault::PageData {
    vault::PageData {
        rel_path: rel.into(),
        title: title.into(),
        book: book.map(str::to_string),
        body: body.into(),
        tags: vec![],
        modified: 0,
    }
}

#[test]
fn upsert_then_search_finds_by_title() {
    let (_t, mut idx) = fresh_index();
    idx.upsert_page(&page("a.md", "Lemon tree", "repotting plan", None))
        .unwrap();
    idx.upsert_page(&page("b.md", "Reading queue", "summer books", None))
        .unwrap();
    let hits = idx.search("lemon", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].rel_path, "a.md");
}

#[test]
fn search_respects_body_text() {
    let (_t, mut idx) = fresh_index();
    idx.upsert_page(&page(
        "a.md",
        "Daily",
        "the quick brown fox jumps",
        Some("Daily"),
    ))
    .unwrap();
    let hits = idx.search("brown", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].snippet.to_lowercase().contains("brown"));
}

#[test]
fn delete_page_drops_from_search() {
    let (_t, mut idx) = fresh_index();
    idx.upsert_page(&page("a.md", "Lemon", "...", None)).unwrap();
    assert_eq!(idx.search("lemon", 10).unwrap().len(), 1);
    idx.delete_page("a.md").unwrap();
    assert_eq!(idx.search("lemon", 10).unwrap().len(), 0);
}

#[test]
fn list_page_titles_returns_all() {
    let (_t, mut idx) = fresh_index();
    idx.upsert_page(&page("a.md", "Alpha", "x", None)).unwrap();
    idx.upsert_page(&page("b.md", "Beta", "y", None)).unwrap();
    let titles = idx.list_page_titles().unwrap();
    let names: Vec<_> = titles.iter().map(|t| t.title.as_str()).collect();
    assert!(names.contains(&"Alpha"));
    assert!(names.contains(&"Beta"));
    assert_eq!(titles.len(), 2);
}

#[test]
fn backlinks_resolve_by_stem() {
    let (_t, mut idx) = fresh_index();
    idx.upsert_page(&page("Target.md", "Target", "destination", None))
        .unwrap();
    idx.upsert_page(&page(
        "Linker.md",
        "Linker",
        "see [[Target]] and also [[Target|alias]]",
        None,
    ))
    .unwrap();
    let backs = idx.find_backlinks("Target.md").unwrap();
    assert_eq!(backs.len(), 1);
    assert_eq!(backs[0].from_title, "Linker");
}

#[test]
fn upsert_replaces_existing_row() {
    let (_t, mut idx) = fresh_index();
    idx.upsert_page(&page("a.md", "Original", "first body", None))
        .unwrap();
    idx.upsert_page(&page("a.md", "Renamed", "second body", None))
        .unwrap();
    let titles = idx.list_page_titles().unwrap();
    assert_eq!(titles.len(), 1);
    assert_eq!(titles[0].title, "Renamed");
    let hits = idx.search("second", 10).unwrap();
    assert_eq!(hits.len(), 1);
    let stale = idx.search("first", 10).unwrap();
    assert_eq!(stale.len(), 0);
}

#[test]
fn delete_pages_with_prefix_clears_a_book() {
    let (_t, mut idx) = fresh_index();
    idx.upsert_page(&page("Book/a.md", "BA", "x", Some("Book")))
        .unwrap();
    idx.upsert_page(&page("Book/b.md", "BB", "y", Some("Book")))
        .unwrap();
    idx.upsert_page(&page("loose.md", "Loose", "z", None)).unwrap();
    idx.delete_pages_with_prefix("Book/").unwrap();
    let remaining = idx.list_page_titles().unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].title, "Loose");
}
