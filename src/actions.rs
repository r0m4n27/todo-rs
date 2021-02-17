use std::{fs::File, io};

use io::BufReader;

use crate::config::Config;
use crate::project::find_files;
use crate::todo_parser::TodoParser;

pub fn todo_files(conf: &Config) {
    let files = find_files(&conf.root, &conf.filter_fn);

    for path in files {
        println!("{}", path.strip_prefix(&conf.root).unwrap().display())
    }
}

pub fn list_todos(conf: &Config, reported: bool, unreported: bool) {
    let files = find_files(&conf.root, &conf.filter_fn);

    let parser = TodoParser::new(&conf.keywords, reported, unreported).unwrap();

    for path in files {
        let file = File::open(&path).unwrap();
        let buf = BufReader::new(file);

        if let Ok(todos) = parser.parse_file(buf) {
            let relative = path.strip_prefix(&conf.root).unwrap();

            for todo in todos {
                println!("{}:{}", relative.display(), todo)
            }
        }
    }
}
