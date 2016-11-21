use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::{self, sleep};
use std::time::Duration;

pub fn init_sync(git_path: PathBuf, index_path: &String, port: u16, interval: Duration) {
    let index_path = index_path.clone();
    git_sync(&git_path, &index_path, port);
    thread::spawn(move || loop {
        sleep(interval);
        git_sync(&git_path, &index_path, port);
    });
}

pub fn git_sync(git_path: &PathBuf, index_path: &String, port: u16) {
    debug!("Syncing git repo at {} with {}",
           git_path.to_str().unwrap(),
           index_path);
    let mut repo_path = git_path.clone();
    repo_path.push(".git");
    debug!("Repo path is {:?}", repo_path);
    let status = if repo_path.exists() {
        match Command::new("git")
            .arg("pull")
            .arg("-q")
            .arg("--rebase")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .current_dir(git_path)
            .status() {
            Ok(s) => Some(s),
            Err(e) => {
                warn!("Error pulling: {:?}", e);
                return;
            }
        }
    } else {
        match Command::new("git")
            .arg("clone")
            .arg("-qq")
            .arg(index_path)
            .arg(&git_path)
            .current_dir(git_path)
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .status() {
            Ok(s) => {
                Command::new("git")
                    .arg("config")
                    .arg("commit.gpgsign")
                    .arg("false")
                    .stderr(Stdio::null())
                    .stdout(Stdio::null())
                    .current_dir(git_path)
                    .status()
                    .unwrap();
                Some(s)
            }
            Err(_) => return,
        }
    };
    let mut config_path = git_path.clone();
    config_path.push("config.json");
    if let Ok(mut f) = File::create(config_path) {
        let config = format!("{{
  \"dl\": \"http://localhost:{0}/api/v1/crates\",
  \"api\": \"http://localhost:{0}/\"
}}
",
                             port);
        let _ = f.write(&config.as_bytes());
        Command::new("git")
            .arg("commit")
            .arg("-q")
            .arg("-a")
            .arg("-m 'Updating config.json'")
            .arg("--no-gpg-sign")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .current_dir(git_path)
            .status()
            .unwrap();
        let mut export = git_path.clone();
        export.push(".git");
        export.push("git-daemon-export-ok");
        let _ = File::create(export);
    } else {
        warn!("\tHad a problem modifying the config.json")
    }
    if let Some(status) = status {
        if status.success() {
            trace!("Successfully synced");
            return;
        } else {
            warn!("Command was not a success");
        }
    }
    warn!("Failed to update index");
}
