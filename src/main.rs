use crate::project::{base_dir, find_files};

mod project;

fn main() {
    let base_result = base_dir().unwrap();

    println!("{}", base_result.display());

    let files = find_files(&base_result, |path| {
        !path.to_string_lossy().contains("target") && !path.to_string_lossy().contains(".git")
    });
    for path in files.iter() {
        println!("{}", path.display())
    }
}
