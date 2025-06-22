use std::env;
use std::io::{self, Error};
use std::path::{Path, PathBuf};

use git2::{build::RepoBuilder, BranchType, Cred, FetchOptions, RemoteCallbacks, Repository};
use tempfile::TempDir;

const ENV_SSH_PASSPHRASE: &str = "SSH_PASSPHRASE";

pub(crate) struct RepoManager {
    pub(crate) temp_dir: TempDir,
    pub(crate) ssh_key_path: Option<PathBuf>,
}

impl RepoManager {
    pub(crate) fn new() -> Result<Self, io::Error> {
        TempDir::new()
            .map(|temp_dir| Self {
                temp_dir,
                ssh_key_path: None,
            })
            .map_err(|_| Error::other("Failed to create temp dir"))
    }

    pub(crate) fn new_with_key(ssh_key_path: PathBuf) -> Result<Self, io::Error> {
        TempDir::new()
            .map(|temp_dir| Self {
                temp_dir,
                ssh_key_path: Some(ssh_key_path),
            })
            .map_err(|_| Error::other("Failed to create temp dir"))
    }

    pub(crate) fn clone_repo(
        &self,
        repo_url: &str,
        dest_name: &str,
    ) -> Result<PathBuf, git2::Error> {
        let dest_path = self.temp_dir.path().join(dest_name);

        let mut builder = RepoBuilder::new();

        // Only set up SSH authentication if we have an SSH key
        if let Some(ref ssh_key_path) = self.ssh_key_path {
            // Check if the URL requires SSH authentication
            if repo_url.starts_with("git@") {
                let ssh_key_path = ssh_key_path.clone();
                let passphrase = env::var(ENV_SSH_PASSPHRASE).ok();

                let mut callbacks = RemoteCallbacks::new();
                callbacks.credentials(move |_url, username_from_url, _allowed_types| {
                    match username_from_url {
                        Some(username) => {
                            Cred::ssh_key(username, None, &ssh_key_path, passphrase.as_deref())
                        }
                        None => Err(git2::Error::from_str(
                            "❌ Username for SSH authentication is missing",
                        )),
                    }
                });

                let mut fetch_options = FetchOptions::new();
                fetch_options.remote_callbacks(callbacks);
                builder.fetch_options(fetch_options);
            }
        } else if repo_url.starts_with("git@") {
            return Err(git2::Error::from_str(
                "❌ SSH URL requires an SSH key. Please provide --ssh-key argument or use HTTPS URL."
            ));
        }

        match builder.clone(repo_url, &dest_path) {
            Ok(_) => {
                println!("✅ Successfully cloned {repo_url}");
                Ok(dest_path)
            }
            Err(err) => Err(git2::Error::from_str(&format!(
                "❌ Failed to clone repository: {err}"
            ))),
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

pub(crate) fn validate_ssh_key_path(ssh_key_path: &Path) -> Result<(), git2::Error> {
    if !ssh_key_path.exists() {
        return Err(git2::Error::from_str(
            "SSH key not found. Please check the path.",
        ));
    }

    println!("✅ SSH key found: moving forward with cloning.");
    Ok(())
}
