// Phase 13 — in-process git sync via libgit2.
//
// Scope is deliberately small: open/init the vault as a repo, set a remote,
// and do plain pull/push of one configured branch. Pull does fast-forward
// when possible and a normal merge otherwise; on conflict we leave the
// working tree with conflict markers and surface the conflicted files —
// no automatic resolution.
//
// Auth: HTTPS personal-access-token (token stored in the OS keychain) or
// SSH-agent. Custom SSH key paths are out of scope for v1.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use git2::{
    build::CheckoutBuilder, AutotagOption, Cred, CredentialType, FetchOptions, PushOptions,
    RemoteCallbacks, Repository, Signature, StatusOptions,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum AuthKind {
    None,
    Token,
    SshAgent,
}

impl AuthKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "token" => AuthKind::Token,
            "ssh-agent" => AuthKind::SshAgent,
            _ => AuthKind::None,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct DirtyFile {
    pub path: String,
    pub state: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct GitStatus {
    pub initialized: bool,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub dirty: Vec<DirtyFile>,
    pub conflicted: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct PullResult {
    pub kind: String, // "up-to-date" | "fast-forward" | "merged" | "conflicts"
    pub conflicted: Vec<String>,
}

pub fn ensure_repo_with_remote(vault_root: &Path, remote_url: &str) -> Result<()> {
    let repo = match Repository::open(vault_root) {
        Ok(r) => r,
        Err(_) => Repository::init(vault_root).context("git init")?,
    };
    match repo.find_remote("origin") {
        Ok(r) => {
            if r.url() != Some(remote_url) {
                drop(r);
                repo.remote_set_url("origin", remote_url)?;
            }
        }
        Err(_) => {
            repo.remote("origin", remote_url)?;
        }
    }
    Ok(())
}

pub fn status(vault_root: &Path) -> Result<GitStatus> {
    let repo = match Repository::open(vault_root) {
        Ok(r) => r,
        Err(_) => {
            return Ok(GitStatus {
                initialized: false,
                branch: None,
                remote_url: None,
                ahead: 0,
                behind: 0,
                dirty: vec![],
                conflicted: vec![],
            });
        }
    };

    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));
    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()));

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).include_ignored(false);
    let statuses = repo.statuses(Some(&mut opts))?;

    let mut dirty: Vec<DirtyFile> = Vec::new();
    let mut conflicted: Vec<String> = Vec::new();
    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        if path.is_empty() {
            continue;
        }
        let s = entry.status();
        if s.is_conflicted() {
            conflicted.push(path);
            continue;
        }
        let label = if s.is_index_new() || s.is_wt_new() {
            "added"
        } else if s.is_index_deleted() || s.is_wt_deleted() {
            "deleted"
        } else if s.is_index_modified() || s.is_wt_modified() {
            "modified"
        } else if s.is_index_renamed() || s.is_wt_renamed() {
            "renamed"
        } else {
            continue;
        };
        dirty.push(DirtyFile {
            path,
            state: label.to_string(),
        });
    }

    let (ahead, behind) = match (branch.as_deref(), repo.head().ok()) {
        (Some(br), Some(head)) => {
            let upstream = format!("refs/remotes/origin/{}", br);
            match (head.target(), repo.refname_to_id(&upstream)) {
                (Some(local), Ok(remote)) => {
                    repo.graph_ahead_behind(local, remote).unwrap_or((0, 0))
                }
                _ => (0, 0),
            }
        }
        _ => (0, 0),
    };

    Ok(GitStatus {
        initialized: true,
        branch,
        remote_url,
        ahead,
        behind,
        dirty,
        conflicted,
    })
}

fn install_credentials<'cb>(
    cb: &mut RemoteCallbacks<'cb>,
    auth: AuthKind,
    token: Option<String>,
) {
    cb.credentials(move |_url, username, allowed| match auth {
        AuthKind::Token => {
            let t = token.clone().unwrap_or_default();
            // GitHub-style PAT: any non-empty username works; password is the
            // token. We use "git" to play well with most forges.
            Cred::userpass_plaintext("git", &t)
        }
        AuthKind::SshAgent => {
            if allowed.contains(CredentialType::SSH_KEY) {
                Cred::ssh_key_from_agent(username.unwrap_or("git"))
            } else if allowed.contains(CredentialType::USERNAME) {
                Cred::username(username.unwrap_or("git"))
            } else {
                Err(git2::Error::from_str(
                    "no compatible auth method offered by remote",
                ))
            }
        }
        AuthKind::None => Err(git2::Error::from_str(
            "git auth is not configured (set token or ssh-agent in settings)",
        )),
    });
}

pub fn pull(
    vault_root: &Path,
    branch: &str,
    auth: AuthKind,
    token: Option<String>,
) -> Result<PullResult> {
    let repo = Repository::open(vault_root)?;
    let mut remote = repo.find_remote("origin")?;
    {
        let mut cb = RemoteCallbacks::new();
        install_credentials(&mut cb, auth, token);
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        fo.download_tags(AutotagOption::None);
        remote.fetch(&[branch], Some(&mut fo), None)?;
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.is_up_to_date() {
        return Ok(PullResult {
            kind: "up-to-date".into(),
            conflicted: vec![],
        });
    }

    let local_refname = format!("refs/heads/{}", branch);

    if analysis.is_fast_forward() {
        match repo.find_reference(&local_refname) {
            Ok(mut r) => {
                r.set_target(fetch_commit.id(), "skein: fast-forward")?;
            }
            Err(_) => {
                // Branch doesn't exist locally yet — create it pointing at FETCH_HEAD.
                repo.reference(&local_refname, fetch_commit.id(), true, "skein: create branch")?;
            }
        }
        repo.set_head(&local_refname)?;
        repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
        return Ok(PullResult {
            kind: "fast-forward".into(),
            conflicted: vec![],
        });
    }

    if analysis.is_normal() {
        let head_ref = repo.head()?;
        let head_commit_id = head_ref
            .target()
            .ok_or_else(|| anyhow!("HEAD is not a direct reference"))?;
        let local_tree = repo.find_commit(head_commit_id)?.tree()?;
        let remote_tree = repo.find_commit(fetch_commit.id())?.tree()?;
        let base_id = repo.merge_base(head_commit_id, fetch_commit.id())?;
        let ancestor_tree = repo.find_commit(base_id)?.tree()?;
        let mut idx = repo.merge_trees(&ancestor_tree, &local_tree, &remote_tree, None)?;

        if idx.has_conflicts() {
            // Materialize conflicts in the working tree with diff3-style markers,
            // then bail without committing — user resolves manually.
            let mut co = CheckoutBuilder::new();
            co.allow_conflicts(true).conflict_style_diff3(true);
            repo.checkout_index(Some(&mut idx), Some(&mut co))?;
            let conflicted: Vec<String> = idx
                .conflicts()?
                .filter_map(|c| c.ok())
                .filter_map(|c| {
                    c.our
                        .or(c.their)
                        .or(c.ancestor)
                        .and_then(|e| std::str::from_utf8(&e.path).ok().map(|s| s.to_string()))
                })
                .collect();
            return Ok(PullResult {
                kind: "conflicts".into(),
                conflicted,
            });
        }

        let oid = idx.write_tree_to(&repo)?;
        let tree = repo.find_tree(oid)?;
        let sig = repo
            .signature()
            .or_else(|_| Signature::now("Skein", "skein@local"))?;
        let local_commit = repo.find_commit(head_commit_id)?;
        let remote_commit = repo.find_commit(fetch_commit.id())?;
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Merge from origin",
            &tree,
            &[&local_commit, &remote_commit],
        )?;
        repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
        return Ok(PullResult {
            kind: "merged".into(),
            conflicted: vec![],
        });
    }

    Err(anyhow!("unhandled merge analysis"))
}

pub fn push(
    vault_root: &Path,
    branch: &str,
    auth: AuthKind,
    token: Option<String>,
) -> Result<()> {
    let repo = Repository::open(vault_root)?;
    let mut remote = repo.find_remote("origin")?;
    let mut cb = RemoteCallbacks::new();
    install_credentials(&mut cb, auth, token);
    let mut po = PushOptions::new();
    po.remote_callbacks(cb);
    let refspec = format!("refs/heads/{0}:refs/heads/{0}", branch);
    remote.push(&[refspec.as_str()], Some(&mut po))?;
    Ok(())
}

/// Stage everything dirty + commit with the given message. Used by the
/// "save & push" UX so users don't need a separate commit step.
pub fn commit_all(vault_root: &Path, message: &str) -> Result<bool> {
    let repo = Repository::open(vault_root)?;
    let mut idx = repo.index()?;
    idx.add_all(["*"], git2::IndexAddOption::DEFAULT, None)?;
    idx.write()?;
    let oid = idx.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let sig = repo
        .signature()
        .or_else(|_| Signature::now("Skein", "skein@local"))?;
    let parent = repo.head().ok().and_then(|h| h.target()).and_then(|id| repo.find_commit(id).ok());
    match parent {
        Some(p) => {
            // Skip empty commits.
            if p.tree()?.id() == tree.id() {
                return Ok(false);
            }
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&p])?;
        }
        None => {
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?;
        }
    }
    Ok(true)
}
