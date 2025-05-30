use std::env;
use std::io::{self, Error};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use git2::{build::RepoBuilder, BranchType, Cred, FetchOptions, RemoteCallbacks, Repository};
use tempfile::TempDir;

pub(crate) struct RepoManager {
    pub(crate) temp_dir: TempDir,
}

impl RepoManager {
    pub(crate) fn new() -> Result<Self, io::Error> {
        TempDir::new()
            .map(|temp_dir| Self { temp_dir })
            .map_err(|_| Error::other("Failed to create temp dir"))
    }

    pub(crate) fn clone_repo(
        &self,
        repo_url: &str,
        dest_name: &str,
    ) -> Result<PathBuf, git2::Error> {
        let dest_path = self.temp_dir.path().join(dest_name);

        let mut callbacks = RemoteCallbacks::new();
        let mut attempted = false;

        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            if attempted {
                return Err(git2::Error::from_str("❌ SSH authentication failed"));
            }
            attempted = true;

            match username_from_url {
                Some(username) => Cred::ssh_key_from_agent(username),
                None => Err(git2::Error::from_str(
                    "❌ Username for SSH authentication is missing",
                )),
            }
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);

        match builder.clone(repo_url, &dest_path) {
            Ok(_) => {
                println!("✅ Successfully cloned {repo_url}");
                Ok(dest_path)
            }
            Err(_) => Err(git2::Error::from_str("❌ Failed to clone repository.")),
        }
    }

    pub(crate) fn checkout(repo_path: &Path, reference: &str) -> Result<(), git2::Error> {
        let repo = Repository::open(repo_path)?;
        let obj = repo.revparse_single(reference)?;

        repo.checkout_tree(&obj, None)?;

        if repo.find_branch(reference, BranchType::Local).is_ok() {
            repo.set_head(&format!("refs/heads/{reference}"))?;
        } else if repo
            .find_reference(&format!("refs/tags/{reference}"))
            .is_ok()
        {
            repo.set_head(&format!("refs/tags/{reference}"))?;
        } else {
            repo.set_head_detached(obj.id())?;
        }

        Ok(())
    }
}

pub(crate) fn setup_ssh_agent(ssh_key_path: &Path) -> Result<(), git2::Error> {
    if env::var("SSH_AUTH_SOCK").is_err() {
        let output = Command::new("ssh-agent")
            .stdout(Stdio::piped())
            .output()
            .map_err(|_| git2::Error::from_str("Failed to start ssh-agent"))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if let Some((key, value)) = line.split_once('=') {
                if key == "SSH_AUTH_SOCK" || key == "SSH_AGENT_PID" {
                    let value = value.trim_end_matches(';');
                    env::set_var(key, value);
                }
            }
        }
    }

    if !ssh_key_path.exists() {
        return Err(git2::Error::from_str(
            "SSH key not found at the specified path.",
        ));
    }

    let ssh_add_output = Command::new("ssh-add")
        .arg(ssh_key_path)
        .output()
        .map_err(|_| git2::Error::from_str("Failed to run ssh-add"))?;

    if !ssh_add_output.status.success() {
        return Err(git2::Error::from_str("Failed to add SSH key to agent."));
    }

    let ssh_test = Command::new("ssh")
        .arg("-T")
        .arg("git@github.com")
        .output()
        .map_err(|_| git2::Error::from_str("Failed to execute SSH command"))?;

    let ssh_error = String::from_utf8_lossy(&ssh_test.stderr);
    if ssh_error.contains("successfully authenticated") {
        return Ok(());
    }

    Err(git2::Error::from_str("❌ SSH authentication test failed."))
}
