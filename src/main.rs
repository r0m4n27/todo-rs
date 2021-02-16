use std::{fs::File, io::BufReader};

use config::Config;
use project::find_files;
use todo_parser::TodoParser;

mod config;
mod project;
mod todo;
mod todo_parser;

fn main() {
    let conf = Config::default().unwrap();

    let files = find_files(&conf.root, &conf.filter_fn);
    let parser = TodoParser::new(&conf.keywords, true, true).unwrap();

    for path in files.iter() {
        let relative = path.strip_prefix(&conf.root).unwrap();
        let file = File::open(path).unwrap();
        let buf = BufReader::new(file);

        if let Ok(todos) = parser.parse_file(buf) {
            for todo in todos {
                println!("{}:{}", relative.display(), todo)
            }
        }
    }
}
