use std::{fs::read_to_string, path::Path};

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub enum IgnoreMode {
    Blacklist,
    Whitelist,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum Backend {
    Gitea,
    Github,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RawConfig {
    pub ignore_mode: Option<IgnoreMode>,
    pub backend: Option<Backend>,
    pub patterns: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub user: Option<String>,
    pub repo: Option<String>,
    pub token: Option<String>,
    pub url: Option<String>,
}

impl Default for RawConfig {
    fn default() -> Self {
        RawConfig {
            ignore_mode: None,
            patterns: None,
            keywords: None,
            backend: None,
            user: None,
            repo: None,
            token: None,
            url: None,
        }
    }
}

impl RawConfig {
    pub fn from_path(path: &Path) -> Self {
        read_to_string(path)
            .ok()
            .and_then(|t| serde_yaml::from_str(&t).ok())
            .unwrap_or(Default::default())
    }

    pub fn merge(global: Self, local: Self) -> Self {
        let mut same_mode = true;

        let ignore_mode = merge_fn(
            global.ignore_mode,
            local.ignore_mode,
            IgnoreMode::Blacklist,
            |g_mode, l_mode| {
                if g_mode != l_mode {
                    same_mode = false;
                }

                l_mode
            },
        );

        let patterns = merge_fn(global.patterns, local.patterns, vec![], |mut g, mut l| {
            if same_mode {
                l.append(&mut g);
            }
            l
        });

        let keywords = merge_fn(
            global.keywords,
            local.keywords,
            vec!["TODO".to_string()],
            |_, l| l,
        );

        let backend = merge(global.backend, local.backend);
        let user = merge(global.user, local.user);
        let repo = merge(global.repo, local.repo);
        let token = merge(global.token, local.token);
        let url = merge(global.url, local.url);

        RawConfig {
            ignore_mode,
            patterns,
            keywords,
            backend,
            user,
            repo,
            token,
            url,
        }
    }
}

fn merge<T>(global: Option<T>, local: Option<T>) -> Option<T> {
    match (global, local) {
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(_), Some(local)) => Some(local),
    }
}

fn merge_fn<T, F>(global: Option<T>, local: Option<T>, default: T, mut merge_fn: F) -> Option<T>
where
    F: FnMut(T, T) -> T,
{
    match (global, local) {
        (None, None) => Some(default),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(global), Some(local)) => Some(merge_fn(global, local)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod merge {
        use super::*;

        #[test]
        fn merge_none() {
            let expected: Option<i32> = None;

            assert_eq!(expected, merge(None, None))
        }

        #[test]
        fn merge_one() {
            assert_eq!(Some(123), merge(Some(123), None));
            assert_eq!(Some(123), merge(None, Some(123)))
        }

        #[test]
        fn merge_local() {
            assert_eq!(Some(123), merge(Some(789), Some(123)))
        }
    }

    mod merge_fn {
        use super::*;

        #[test]
        fn merge_none() {
            assert_eq!(Some(123), merge_fn(None, None, 123, |_, l| l))
        }

        #[test]
        fn merge_one() {
            let func = |_, l| l;

            assert_eq!(Some(123), merge_fn(Some(123), None, 456, func));
            assert_eq!(Some(123), merge_fn(None, Some(123), 456, func))
        }

        #[test]
        fn merge_both() {
            let func = |_, l| l;

            assert_eq!(Some(123), merge_fn(Some(789), Some(123), 456, func))
        }
    }

    mod raw_merge {

        use super::*;

        #[test]
        fn merge_empty() {
            assert_eq!(
                RawConfig {
                    ignore_mode: Some(IgnoreMode::Blacklist),
                    patterns: Some(vec![]),
                    keywords: Some(vec!["TODO".to_owned()]),
                    ..Default::default()
                },
                RawConfig::merge(Default::default(), Default::default())
            )
        }

        #[test]
        fn merge_priority() {
            let local = RawConfig {
                ignore_mode: Some(IgnoreMode::Whitelist),
                keywords: Some(vec!["TODO".to_string()]),
                patterns: Some(vec![".git".to_string()]),
                ..Default::default()
            };

            let global: RawConfig = RawConfig {
                ignore_mode: Some(IgnoreMode::Blacklist),
                keywords: Some(vec!["FIXME".to_string(), "BUG".to_string()]),
                patterns: Some(vec!["target".to_string()]),
                ..Default::default()
            };

            assert_eq!(
                RawConfig {
                    ignore_mode: Some(IgnoreMode::Whitelist),
                    patterns: Some(vec![".git".to_string()]),
                    keywords: Some(vec!["TODO".to_string()]),
                    ..Default::default()
                },
                RawConfig::merge(global, local)
            )
        }

        #[test]
        fn merge_add_patterns() {
            let local = RawConfig {
                ignore_mode: Some(IgnoreMode::Blacklist),
                patterns: Some(vec!["123".to_string(), "456".to_string()]),
                ..Default::default()
            };

            let global = RawConfig {
                ignore_mode: Some(IgnoreMode::Blacklist),
                patterns: Some(vec!["789".to_string()]),
                ..Default::default()
            };

            assert_eq!(
                RawConfig {
                    ignore_mode: Some(IgnoreMode::Blacklist),
                    patterns: Some(vec![
                        "123".to_string(),
                        "456".to_string(),
                        "789".to_string()
                    ]),
                    keywords: Some(vec!["TODO".to_string()]),
                    ..Default::default()
                },
                RawConfig::merge(global, local)
            )
        }
    }
}
