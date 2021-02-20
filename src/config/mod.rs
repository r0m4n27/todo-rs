use regex::RegexSet;
use std::path::PathBuf;

use thiserror::Error;

use self::raw::{Backend, IgnoreMode, RawConfig};
use crate::{
    api::{gitea::Gitea, github::Github, Api},
    Result,
};

mod raw;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Can't compile Paterns!")]
    Pattern,

    #[error("Config misses {0}!")]
    MissingValue(String),
}

pub struct Config {
    pub keywords: Vec<String>,
    pub root: PathBuf,
    pub filter_fn: Box<dyn Fn(&PathBuf) -> bool>,
    pub api: Box<dyn Api>,
}

impl Config {
    pub async fn default(root: PathBuf) -> Result<Config> {
        let raw = RawConfig::merge(
            RawConfig::from_path(&dirs::config_dir().unwrap().join("todo.yml")),
            RawConfig::from_path(&root.join(".todo.yml")),
        );

        if let Ok(patterns) = RegexSet::new(raw.patterns.unwrap()) {
            let api = create_api(raw.backend, raw.user, raw.repo, raw.token, raw.url).await?;

            Ok(Config {
                keywords: raw.keywords.unwrap(),
                root,
                filter_fn: create_filter_fn(raw.ignore_mode.unwrap(), patterns),
                api,
            })
        } else {
            Err(ConfigError::Pattern.into())
        }
    }
}

async fn create_api(
    backend: Option<Backend>,
    user: Option<String>,
    repo: Option<String>,
    token: Option<String>,
    url: Option<String>,
) -> Result<Box<dyn Api>> {
    let backend = backend.ok_or(ConfigError::MissingValue("backend".to_owned()))?;

    let user = user.ok_or(ConfigError::MissingValue("user".to_owned()))?;
    let repo = repo.ok_or(ConfigError::MissingValue("repo".to_owned()))?;
    let token = token.ok_or(ConfigError::MissingValue("token".to_owned()))?;

    match backend {
        Backend::Gitea => {
            let mut url = url.ok_or(ConfigError::MissingValue("url".to_owned()))?;
            url.push_str("/api/v1");

            Ok(Box::new(Gitea::new(&url, token, &user, &repo).await?))
        }
        Backend::Github => Ok(Box::new(Github::new(user, repo, token.clone()).await?)),
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
