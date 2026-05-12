//! Fetch a Pull Request from AtomGit and format its details + changed files
//! into a text report. Used by the `/pr` slash command in the TUI.

use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::client::Client;
use super::models::{PrFile, PullRequest};
use super::url::{detect_cwd_atomgit_repo, PrRef, RepoRef};

/// Fetches a PR from AtomGit, validates the CWD repo matches, and returns
/// a formatted text report with PR metadata + list of changed files.
pub fn get_pr(pr_url: &str, working_dir: &Path) -> Result<String> {
    let pr_ref = PrRef::parse(pr_url)
        .map_err(|e| anyhow!("invalid PR URL: {}", e))?;

    // Validate that the CWD's git origin matches the PR's repo.
    // If the CWD has no atomgit.com remote, allow the request (the user
    // may be reading a PR from outside the repo — still useful).
    let pr_repo = RepoRef::from(&pr_ref);
    match detect_cwd_atomgit_repo(working_dir) {
        Ok(Some(cwd_repo)) => {
            if !cwd_repo.matches(&pr_repo) {
                return Err(anyhow!(
                    "PR repo ({}/{}) does not match the current working directory ({}/{})",
                    pr_ref.owner, pr_ref.repo, cwd_repo.owner, cwd_repo.repo,
                ));
            }
        }
        Ok(None) => { /* no atomgit.com remote — skip validation */ }
        Err(_) => {
            // Non-fatal: proceed (the API call will fail with
            // its own message if there's a real auth/network problem).
        }
    }

    let client = Client::from_stored_auth()
        .context("failed to create AtomGit API client (try `atomcode login` first)")?;

    let pr = client
        .get_pull_request(&pr_ref)
        .context("failed to fetch pull request details")?;

    let files = client
        .get_pull_request_files(&pr_ref);

    Ok(format_pr_report(&pr, &files))
}

fn format_pr_report(pr: &PullRequest, files: &[PrFile]) -> String {
    let mut out = String::new();

    // Title bar
    out.push_str(&format!(
        "# Pull Request #{}: {}\n",
        pr.number, pr.title
    ));
    out.push_str(&format!("State: **{}**", pr.state));
    if let Some(ref user) = pr.user {
        out.push_str(&format!(" | Author: **{}**", user.login));
    }
    if let Some(ref base) = pr.base_label {
        out.push_str(&format!(" | Base: `{}`", base));
    }
    if let Some(ref head) = pr.head_label {
        out.push_str(&format!(" | Head: `{}`", head));
    }
    out.push('\n');

    if let Some(ref url) = pr.html_url {
        out.push_str(&format!("URL: {}\n", url));
    }

    // Body
    if let Some(ref body) = pr.body {
        if !body.trim().is_empty() {
            out.push_str("\n## Description\n\n");
            out.push_str(body.trim());
            out.push('\n');
        }
    }

    // Changed files
    out.push_str(&format!("\n## Changed Files ({})\n\n", files.len()));
    if files.is_empty() {
        out.push_str("_(no file changes found)_\n");
    } else {
        for f in files {
            let status_icon = match f.status.as_str() {
                "added" => "🟢",
                "removed" => "🔴",
                "modified" => "🟡",
                "renamed" => "🔵",
                _ => "⚪",
            };
            out.push_str(&format!(
                "{} {}  (+{} / -{})  {}\n",
                status_icon, f.status, f.additions, f.deletions, f.filename
            ));
        }
    }

    out
}
