use std::fs::read_dir;
use std::path::PathBuf;

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

pub fn find_files<F>(root: &PathBuf, filter_fn: &F) -> Vec<PathBuf>
where
    F: Fn(&PathBuf) -> bool,
{
    if root.is_dir() {
        read_dir(root)
            .unwrap()
            .filter_map(|r| r.ok().map(|d| d.path()))
            .filter(filter_fn)
            .flat_map(|path| {
                if path.is_dir() {
                    find_files(&path, filter_fn)
                } else {
                    vec![path]
                }
            })
            .collect()
    } else {
        vec![]
    }
}

pub fn add_to_git() {
    let _ = Exec::shell("git add -a");
}
