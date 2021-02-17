use regex::RegexSet;
use std::path::PathBuf;

use self::raw::{IgnoreMode, RawConfig};
use crate::api::Api;
use crate::project::base_dir;

mod raw;

pub struct Config<'a> {
    pub keywords: Vec<String>,
    pub root: PathBuf,
    pub filter_fn: Box<dyn Fn(&PathBuf) -> bool>,
    pub api: &'a dyn Api,
}

impl<'a> Config<'a> {
    pub fn default(dummy_api: &'a impl Api) -> Result<Self, &'static str> {
        let root = base_dir()?;
        let raw = RawConfig::merge(
            RawConfig::from_path(&dirs::config_dir().unwrap().join("todo.yml")),
            RawConfig::from_path(&root.join(".todo.yml")),
        );

        if let Ok(patterns) = RegexSet::new(raw.patterns.unwrap()) {
            Ok(Config {
                keywords: raw.keywords.unwrap(),
                root,
                filter_fn: create_filter_fn(raw.ignore_mode.unwrap(), patterns),
                api: dummy_api,
            })
        } else {
            Err("Can't compile patterns!")
        }
    }
}

fn create_filter_fn(mode: IgnoreMode, regex_set: RegexSet) -> Box<dyn Fn(&PathBuf) -> bool> {
    match mode {
        IgnoreMode::Blacklist => {
            Box::new(move |path: &PathBuf| !regex_set.is_match(&path.to_string_lossy()))
        }
        IgnoreMode::Whitelist => {
            Box::new(move |path: &PathBuf| regex_set.is_match(&path.to_string_lossy()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blacklist_filter() {
        let filter = create_filter_fn(
            IgnoreMode::Blacklist,
            RegexSet::new(&["hallo(.*)"]).unwrap(),
        );

        assert_eq!(false, filter(&PathBuf::from("hallo123")))
    }

    #[test]
    fn whitelist_filter() {
        let filter = create_filter_fn(
            IgnoreMode::Whitelist,
            RegexSet::new(&["hallo(.*)"]).unwrap(),
        );

        assert_eq!(true, filter(&PathBuf::from("hallo123")))
    }
}
