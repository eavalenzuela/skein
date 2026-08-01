// Path-containment regression suite.
//
// Every case here is a way a hostile vault (an imported archive, a cloned
// git remote, a crafted IPC argument) previously reached outside the vault
// root. They all run against real tempdirs so the symlink cases exercise
// the actual filesystem behaviour rather than a mocked one.

use std::fs;

use skein_lib::*;
use tempfile::TempDir;

fn fresh_vault() -> (TempDir, vault::Vault) {
    let dir = TempDir::new().expect("tempdir");
    let v = vault::Vault::from_path(dir.path().to_path_buf()).expect("vault");
    (dir, v)
}

/// A vault plus a sibling directory that stands in for "the rest of the
/// user's home" — anything landing there is an escape.
fn vault_with_outside() -> (TempDir, vault::Vault, std::path::PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let root = dir.path().join("vault");
    let outside = dir.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let v = vault::Vault::from_path(root).expect("vault");
    (dir, v, outside)
}

#[test]
fn rejects_traversal_before_touching_the_filesystem() {
    for bad in [
        "../escape.md",
        "../../escape.md",
        "Book/../../escape.md",
        "/etc/passwd",
        "",
    ] {
        assert!(
            vault::check_rel_path(bad).is_err(),
            "expected `{bad}` to be rejected"
        );
    }
    for ok in ["note.md", "Book/note.md", "./note.md"] {
        assert!(
            vault::check_rel_path(ok).is_ok(),
            "expected `{ok}` to be accepted"
        );
    }
}

#[test]
fn rejected_write_creates_no_directories() {
    let (tmp, v, _outside) = vault_with_outside();
    // Previously create_dir_all ran before the containment check, so a
    // refused write still littered directories outside the vault.
    let err = vault::write_page_body(&v, "../../made/up/tree/x.md", "x").unwrap_err();
    assert!(err.to_string().contains("escapes vault root"), "{err}");
    assert!(!tmp.path().join("made").exists());
}

#[test]
fn write_refuses_to_follow_a_symlinked_page() {
    let (_tmp, v, outside) = vault_with_outside();
    let secret = outside.join("secret.md");
    fs::write(&secret, "original\n").unwrap();

    // A git remote can ship a mode-120000 blob that materializes exactly
    // like this: a note in the vault pointing at a file outside it.
    #[cfg(unix)]
    std::os::unix::fs::symlink(&secret, v.root.join("Note.md")).unwrap();
    #[cfg(windows)]
    let _ = std::os::windows::fs::symlink_file(&secret, v.root.join("Note.md"));

    let err = vault::write_page_body(&v, "Note.md", "clobbered\n").unwrap_err();
    assert!(err.to_string().contains("symlink"), "{err}");
    assert_eq!(fs::read_to_string(&secret).unwrap(), "original\n");
}

#[test]
fn read_refuses_to_follow_a_symlinked_page() {
    let (_tmp, v, outside) = vault_with_outside();
    let secret = outside.join("secret.md");
    fs::write(&secret, "private\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&secret, v.root.join("Note.md")).unwrap();
    #[cfg(windows)]
    let _ = std::os::windows::fs::symlink_file(&secret, v.root.join("Note.md"));

    assert!(vault::read_page_body(&v, "Note.md").is_err());
}

#[test]
fn write_refuses_a_symlinked_book_folder() {
    let (_tmp, v, outside) = vault_with_outside();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, v.root.join("Elsewhere")).unwrap();
    #[cfg(windows)]
    let _ = std::os::windows::fs::symlink_dir(&outside, v.root.join("Elsewhere"));

    let err = vault::write_page_body(&v, "Elsewhere/planted.md", "x").unwrap_err();
    assert!(err.to_string().contains("symlink"), "{err}");
    assert!(!outside.join("planted.md").exists());
}

#[test]
fn delete_book_refuses_a_traversing_name() {
    let (_tmp, v, outside) = vault_with_outside();
    fs::write(outside.join("keepme.txt"), "important\n").unwrap();

    // delete_book previously skipped validate_name and went straight to
    // remove_dir_all, so "../outside" deleted a whole tree outside the vault.
    let err = books::delete_book(&v, "../outside", true).unwrap_err();
    assert!(outside.join("keepme.txt").exists(), "{err}");

    assert!(books::delete_book(&v, "/etc", true).is_err());
}

#[test]
fn delete_book_refuses_a_symlinked_folder() {
    let (_tmp, v, outside) = vault_with_outside();
    fs::write(outside.join("keepme.txt"), "important\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&outside, v.root.join("Elsewhere")).unwrap();
    #[cfg(windows)]
    let _ = std::os::windows::fs::symlink_dir(&outside, v.root.join("Elsewhere"));

    assert!(books::delete_book(&v, "Elsewhere", true).is_err());
    assert!(outside.join("keepme.txt").exists());
}

#[test]
fn delete_page_refuses_traversal_and_symlinks() {
    let (_tmp, v, outside) = vault_with_outside();
    let secret = outside.join("secret.md");
    fs::write(&secret, "private\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&secret, v.root.join("Note.md")).unwrap();
    #[cfg(windows)]
    let _ = std::os::windows::fs::symlink_file(&secret, v.root.join("Note.md"));

    assert!(pages::delete_page(&v, "Note.md").is_err());
    assert!(pages::delete_page(&v, "../outside/secret.md").is_err());
    assert!(secret.exists());
}

#[test]
fn create_page_refuses_a_traversing_book() {
    let (_tmp, v, outside) = vault_with_outside();
    assert!(pages::create_page(&v, Some("../outside"), "planted").is_err());
    assert!(!outside.join("planted.md").exists());
}

#[test]
fn move_page_refuses_a_traversing_destination() {
    let (_tmp, v, outside) = vault_with_outside();
    let rel = pages::create_page(&v, None, "mine").unwrap();
    assert!(pages::move_page(&v, &rel, Some("../outside")).is_err());
    assert!(!outside.join("mine.md").exists());
    assert!(v.root.join("mine.md").is_file());
}

#[test]
fn attachments_only_accept_real_images() {
    let (_tmp, v) = fresh_vault();
    let rel = pages::create_page(&v, None, "host").unwrap();

    // A caller-chosen extension used to decide what landed on disk; now the
    // bytes do. A shell script claiming to be a PNG is refused outright.
    let err = attachments::save_attachment(&v, &rel, "png", b"#!/bin/sh\nrm -rf ~\n").unwrap_err();
    assert!(err.to_string().contains("not a supported image"), "{err}");

    let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR-ish-body";
    let saved = attachments::save_attachment(&v, &rel, "exe", png).unwrap();
    assert!(
        saved.ends_with(".png"),
        "extension must come from the bytes, got {saved}"
    );
}

#[test]
fn attachments_refuse_a_page_outside_the_vault() {
    let (_tmp, v, outside) = vault_with_outside();
    let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR-ish-body";
    assert!(attachments::save_attachment(&v, "../outside/page.md", "png", png).is_err());
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
}

#[test]
fn git_remote_url_credentials_are_split_out() {
    // A pasted tokenised clone URL must not persist to settings.json or
    // .git/config; the secret goes to the keychain instead.
    let (url, secret) = git_sync::split_url_credentials("https://ghp_abc123@github.com/me/notes.git");
    assert_eq!(url, "https://github.com/me/notes.git");
    assert_eq!(secret.as_deref(), Some("ghp_abc123"));

    let (url, secret) =
        git_sync::split_url_credentials("https://user:s3cret@gitlab.com/me/notes.git");
    assert_eq!(url, "https://gitlab.com/me/notes.git");
    assert_eq!(secret.as_deref(), Some("s3cret"));

    // Plain URLs and scp-style remotes pass through untouched.
    let (url, secret) = git_sync::split_url_credentials("https://github.com/me/notes.git");
    assert_eq!(url, "https://github.com/me/notes.git");
    assert_eq!(secret, None);

    let (url, secret) = git_sync::split_url_credentials("git@github.com:me/notes.git");
    assert_eq!(url, "git@github.com:me/notes.git");
    assert_eq!(secret, None);
}
