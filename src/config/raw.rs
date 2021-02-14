use std::{fs::read_to_string, path::Path};

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub enum IgnoreMode {
    Blacklist,
    Whitelist,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RawConfig {
    pub ignore_mode: Option<IgnoreMode>,
    pub patterns: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
}

impl Default for RawConfig {
    fn default() -> Self {
        RawConfig {
            ignore_mode: None,
            patterns: None,
            keywords: None,
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

        let mode = merge_opt(
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

        let ignores = merge_opt(global.patterns, local.patterns, vec![], |mut g, mut l| {
            if same_mode {
                l.append(&mut g);
            }
            l
        });

        let keywords = merge_opt(
            global.keywords,
            local.keywords,
            vec!["TODO".to_string()],
            |_, l| l,
        );

        RawConfig {
            ignore_mode: Some(mode),
            patterns: Some(ignores),
            keywords: Some(keywords),
        }
    }
}

fn merge_opt<T, F>(global: Option<T>, local: Option<T>, default: T, mut merge_fn: F) -> T
where
    F: FnMut(T, T) -> T,
{
    match (global, local) {
        (None, None) => default,
        (Some(value), None) | (None, Some(value)) => value,
        (Some(global), Some(local)) => merge_fn(global, local),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod merge_opt {
        use super::*;

        #[test]
        fn merge_none() {
            assert_eq!(123, merge_opt(None, None, 123, |_, l| l))
        }

        #[test]
        fn merge_one() {
            let func = |_, l| l;

            assert_eq!(123, merge_opt(Some(123), None, 456, func));
            assert_eq!(123, merge_opt(None, Some(123), 456, func))
        }

        #[test]
        fn merge_both() {
            let func = |_, l| l;

            assert_eq!(123, merge_opt(Some(789), Some(123), 456, func))
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
                    keywords: Some(vec!["TODO".to_string()])
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
            };

            let global: RawConfig = RawConfig {
                ignore_mode: Some(IgnoreMode::Blacklist),
                keywords: Some(vec!["FIXME".to_string(), "BUG".to_string()]),
                patterns: Some(vec!["target".to_string()]),
            };

            assert_eq!(
                RawConfig {
                    ignore_mode: Some(IgnoreMode::Whitelist),
                    patterns: Some(vec![".git".to_string()]),
                    keywords: Some(vec!["TODO".to_string()])
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
                    keywords: Some(vec!["TODO".to_string()])
                },
                RawConfig::merge(global, local)
            )
        }
    }
}
