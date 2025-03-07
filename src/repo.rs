use std::io::{self, Error, ErrorKind};
use std::path::PathBuf;

use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
use tempfile::TempDir;

pub struct RepoManager {
    pub temp_dir: TempDir,
}

impl RepoManager {
    pub fn new() -> Result<Self, io::Error> {
        TempDir::new()
            .map(|temp_dir| Self { temp_dir })
            .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create temp dir: {e}")))
    }

    pub fn clone_repo(&self, repo_url: &str, dest_name: &str) -> Result<PathBuf, git2::Error> {
        let dest_path = self.temp_dir.path().join(dest_name);

        // Set up authentication callbacks
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(
            move |_url, username_from_url, _allowed_types| match username_from_url {
                Some(username) => {
                    println!("Using ssh-agent for authentication with username: {username}");
                    Cred::ssh_key_from_agent(username)
                }
                None => Err(git2::Error::from_str(
                    "Username for SSH authentication is missing",
                )),
            },
        );

        // Configure fetch options
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Use RepoBuilder to clone with authentication
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);

        println!("Cloning repository: {repo_url}");
        match builder.clone(repo_url, &dest_path) {
            Ok(_) => {
                println!("Successfully cloned {repo_url} into {dest_path:?}");
                Ok(dest_path)
            }
            Err(e) => {
                eprintln!("Error cloning {repo_url}: {e}");
                Err(git2::Error::from_str(&format!(
                    "Failed to clone repo {repo_url}: {e}"
                )))
            }
        }
    }
}
