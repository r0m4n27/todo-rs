use config::Config;
use project::find_files;

mod config;
mod project;

fn main() {
    let conf = Config::default().unwrap();

    let files = find_files(&conf.root, &conf.filter_fn);
    for path in files.iter() {
        println!("{}", path.display())
    }
}
