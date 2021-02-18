use std::fs::read_to_string;

use crate::config::Config;
use crate::project::find_files;
use crate::todo_parser::find_todos;

pub fn todo_files(conf: &Config) {
    let files = find_files(&conf.root, &conf.filter_fn);

    for path in files {
        println!("{}", path.strip_prefix(&conf.root).unwrap().display())
    }
}

pub fn list_todos(conf: &Config, reported: bool, unreported: bool) {
    let files = find_files(&conf.root, &conf.filter_fn);

    for path in &files {
        let input = read_to_string(path).unwrap();
        let todos = find_todos(&conf.keywords, &input);

        let relative = path.strip_prefix(&conf.root).unwrap();

        todos
            .into_iter()
            .filter(|t| {
                if !unreported && t.issue_id.is_none() {
                    false
                } else if !reported && t.issue_id.is_some() {
                    false
                } else {
                    true
                }
            })
            .for_each(|t| println!("{}:{}", relative.display(), t));
    }
}
