use std::fs::read_dir;
use std::path::{Path, PathBuf};

use subprocess::Exec;

pub fn base_dir() -> Result<PathBuf, &'static str> {
    let command = Exec::cmd("git").arg("rev-parse").arg("--show-toplevel");

    match command.capture() {
        Ok(data) => {
            if data.exit_status.success() {
                Ok(PathBuf::from(data.stdout_str().trim()))
            } else {
                Err("Not in git repository!")
            }
        }

        Err(_) => Err("Cant find git on the system"),
    }
}

pub fn find_files(root: &Path, filter_fn: fn(&Path) -> bool) -> Vec<PathBuf> {
    if filter_fn(root) {
        if root.is_dir() {
            read_dir(root)
                .unwrap()
                .filter_map(|res| match res {
                    Err(_) => None,
                    Ok(entry) => Some(entry),
                })
                .flat_map(|entry| find_files(&entry.path(), filter_fn))
                .collect()
        } else {
            vec![PathBuf::from(root)]
        }
    } else {
        vec![]
    }
}

pub fn add_to_git() {
    let _ = Exec::shell("git add -a");
}
