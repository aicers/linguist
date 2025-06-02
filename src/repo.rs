use std::env;
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

    /// Clone `repo_url` into a subfolder named `dest_name` under our temp dir,
    /// using SSH key + passphrase read from the env.
    pub(crate) fn clone_repo(
        &self,
        repo_url: &str,
        dest_name: &str,
    ) -> Result<PathBuf, git2::Error> {
        // Build the destination path once, move it into the Ok(...)
        let dest = self.temp_dir.path().join(dest_name);

        // Set up credential callback that uses SSH_KEY_PATH + SSH_KEY_PASSPHRASE
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_url, username_opt, _| {
            let user = username_opt
                .ok_or_else(|| git2::Error::from_str("Username missing for SSH URL"))?;

            // must have set this in setup_ssh_agent()
            let key_path = env::var("SSH_KEY_PATH")
                .map(PathBuf::from)
                .map_err(|_| git2::Error::from_str("SSH_KEY_PATH env var not set"))?;

            // passphrase is optional
            let pass = env::var("SSH_KEY_PASSPHRASE").ok();
            Cred::ssh_key(user, None, &key_path, pass.as_deref())
        });

        // Wire up fetch options
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(callbacks);

        // Clone
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fo);

        builder
            .clone(repo_url, &dest)
            .map_err(|_| git2::Error::from_str("❌ Failed to clone repository."))?;

        println!("✅ Successfully cloned {repo_url}");
        Ok(dest)
    }

    /// Checkout a branch, tag, or SHA in an existing repo.
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

/// Validate that the given key exists, then export its path to `SSH_KEY_PATH`
/// so our credential callback can find it.
pub(crate) fn setup_ssh_agent(ssh_key_path: &Path) -> Result<(), git2::Error> {
    if !ssh_key_path.exists() {
        return Err(git2::Error::from_str(
            "SSH key not found at the specified path.",
        ));
    }
    let key_str = ssh_key_path
        .to_str()
        .ok_or_else(|| git2::Error::from_str("SSH key path is not valid UTF-8"))?;
    env::set_var("SSH_KEY_PATH", key_str);
    Ok(())
}
