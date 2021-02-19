use std::fs::read_dir;
use std::path::PathBuf;

use subprocess::Exec;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Not in git repository!")]
    NotInRepo,
    #[error("Cant find git on the system")]
    GitNotFound,
}

pub fn base_dir() -> Result<PathBuf, ProjectError> {
    let command = Exec::cmd("git").arg("rev-parse").arg("--show-toplevel");

    match command.capture() {
        Ok(data) => {
            if data.exit_status.success() {
                Ok(PathBuf::from(data.stdout_str().trim()))
            } else {
                Err(ProjectError::NotInRepo)
            }
        }

        Err(_) => Err(ProjectError::GitNotFound),
    }
}

pub fn find_files<F>(root: &PathBuf, filter_fn: &F) -> Option<Vec<PathBuf>>
where
    F: Fn(&PathBuf) -> bool,
{
    if root.is_dir() {
        let paths: Vec<_> = read_dir(root)
            .unwrap()
            .filter_map(|r| r.ok().map(|d| d.path()))
            .filter(filter_fn)
            .collect();

        let mut output = Vec::new();

        for path in paths {
            if path.is_dir() {
                output.append(&mut find_files(&path, filter_fn).unwrap())
            } else {
                output.push(path)
            }
        }

        Some(output)
    } else {
        None
    }
}

pub fn add_to_git() {
    let _ = Exec::shell("git add -A").join();
}
