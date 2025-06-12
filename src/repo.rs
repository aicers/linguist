use std::io::{self, Error};
use std::path::{Path, PathBuf};

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

pub(crate) fn validate_ssh_key_path(ssh_key_path: &Path) -> Result<(), git2::Error> {
    if ssh_key_path.exists() {
        Ok(())
    } else {
        Err(git2::Error::from_str(
            "SSH key not found at the specified path.",
        ))
    }
}
