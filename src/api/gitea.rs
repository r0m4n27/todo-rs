use std::collections::HashMap;

use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION},
};
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

use crate::todo::Todo;

use super::{Api, ApiError};

#[derive(Debug, Error)]
pub enum GiteaError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error("{0}")]
    Parse(String),
}

pub struct Gitea<'a> {
    issues_url: String,
    labels: HashMap<String, u64>,
    token: &'a str,
    client: Client,
}

impl<'a> Api for Gitea<'a> {
    fn closed_ids(&self) -> Result<Vec<u32>, ApiError> {
        let mut page = 1;
        let mut output = Vec::new();

        loop {
            let json = self.get_issues(&[("state", "closed"), ("page", &format!("{}", page))])?;
            let mut numbers = parse_numbers(json)?;

            if !numbers.is_empty() {
                page += 1;

                output.append(&mut numbers)
            } else {
                break;
            }
        }

        Ok(output)
    }

    fn report_todo(&self, todo: &mut Todo) -> Result<(), ApiError> {
        let mut json: HashMap<&str, Value> = HashMap::new();
        let comment_str = create_comment_string(todo);

        json.insert("title", json!(todo.title.as_str()));
        json.insert("body", json!(&comment_str));

        if let Some(id) = self.labels.get(&todo.keyword) {
            json.insert("labels", json!(&[id]));
        }

        let response = self.post_todo(&json)?;

        todo.issue_id = Some(parse_issue(response)?);

        Ok(())
    }
}

impl<'a> Gitea<'a> {
    pub fn new(base_url: &str, token: &'a str, user: &str, repo: &str) -> Result<Self, ApiError> {
        let label_url = format!("{}/repos/{}/{}/labels", base_url, user, repo);
        let client = Client::new();

        Ok(Gitea {
            issues_url: format!("{}/repos/{}/{}/issues", base_url, user, repo),
            labels: get_labels(&client, &label_url, token)?,
            token,
            client,
        })
    }

    fn get_issues<T>(&self, query: &T) -> Result<Value, GiteaError>
    where
        T: Serialize + ?Sized,
    {
        self.client
            .get(&self.issues_url)
            .headers(create_header(self.token))
            .query(query)
            .send()?
            .json::<Value>()
            .map_err(|err| GiteaError::Request(err))
    }

    fn post_todo<T>(&self, todo: &T) -> Result<Value, GiteaError>
    where
        T: Serialize + ?Sized,
    {
        self.client
            .post(&self.issues_url)
            .headers(create_header(self.token))
            .json(todo)
            .send()?
            .json::<Value>()
            .map_err(|err| GiteaError::Request(err))
    }
}

fn create_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, format!("token {}", token).parse().unwrap());

    headers
}

fn get_labels(client: &Client, url: &str, token: &str) -> Result<HashMap<String, u64>, GiteaError> {
    let response = client
        .get(url)
        .headers(create_header(token))
        .send()?
        .json::<Value>()?;

    Ok(parse_labels(response)?)
}

fn create_comment_string(todo: &Todo) -> String {
    let mut comment_str = String::new();
    let mut last_str = "";

    for comment in &todo.comments {
        if comment == "" {
            comment_str.push_str("\n")
        } else {
            if last_str != "" {
                comment_str.push_str(" ")
            }

            comment_str.push_str(comment)
        }

        last_str = comment;
    }

    comment_str
}

fn parse_issue(val: Value) -> Result<u32, GiteaError> {
    val.as_object()
        .and_then(|o| o.get("number"))
        .and_then(|v| v.as_u64())
        .and_then(|u| Some(u as u32))
        .ok_or(GiteaError::Parse(
            "Cant't parse requested Issue!".to_owned(),
        ))
}

fn parse_labels(val: Value) -> Result<HashMap<String, u64>, GiteaError> {
    val.as_array()
        .and_then(|a| {
            a.into_iter()
                .map(|v| {
                    v.as_object()
                        .and_then(|o| Some((o.get("name")?, o.get("id")?)))
                        .and_then(|t| Some((t.0.as_str()?.to_owned(), t.1.as_u64()?)))
                })
                .collect::<Option<_>>()
        })
        .ok_or(GiteaError::Parse("Can't parse labels!".to_owned()))
}

fn parse_numbers(val: Value) -> Result<Vec<u32>, GiteaError> {
    val.as_array()
        .and_then(|a| {
            a.into_iter()
                .map(|v| {
                    {
                        v.as_object()
                            .and_then(|o| o.get("number"))
                            .and_then(|v| v.as_u64())
                            .and_then(|u| Some(u as u32))
                    }
                })
                .collect::<Option<_>>()
        })
        .ok_or(GiteaError::Parse("Can't parse closed id's".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_comments_normal() {
        let todo = Todo {
            line: 123,
            prefix: "//".to_owned(),
            keyword: "TODO".to_owned(),
            title: "Something".to_owned(),
            issue_id: None,
            comments: vec!["More".to_owned(), "And More".to_owned()],
        };

        assert_eq!("More And More", create_comment_string(&todo))
    }

    #[test]
    fn create_comments_newline() {
        let todo = Todo {
            line: 123,
            prefix: "//".to_owned(),
            keyword: "TODO".to_owned(),
            title: "Something".to_owned(),
            issue_id: None,
            comments: vec!["More".to_owned(), "".to_owned(), "And More".to_owned()],
        };

        assert_eq!("More\nAnd More", create_comment_string(&todo))
    }

    #[test]
    fn parse_issue_success() {
        let val = json!({
            "number": 123
        });

        if let Ok(issue) = parse_issue(val) {
            assert_eq!(123, issue)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parse_issue_fail() {
        let val = json!({
            "number": "123"
        });

        if let Err(GiteaError::Parse(issue)) = parse_issue(val) {
            assert_eq!("Cant't parse requested Issue!".to_owned(), issue)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parse_labels_success() {
        let val = json!([
            {
                "name": "123",
                "id": 123
            },
            {
                "name": "456",
                "id": 456
            }
        ]);

        if let Ok(map) = parse_labels(val) {
            assert_eq!(&123, map.get("123").unwrap());
            assert_eq!(&456, map.get("456").unwrap())
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parse_labels_fail() {
        let val = json!([
            {
                "name": "123",
            },
            {
                "name": "456",
                "id": 456
            }
        ]);

        if let Err(GiteaError::Parse(issue)) = parse_labels(val) {
            assert_eq!("Can't parse labels!".to_owned(), issue)
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parse_numbers_success() {
        let val = json!([
            {
                "number": 123,
            },
            {
                "number": 456,
            }
        ]);

        if let Ok(numbers) = parse_numbers(val) {
            assert_eq!(vec![123, 456], numbers);
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parse_numbers_fail() {
        let val = json!([
            {
                "name": "123",
            },
            {
                "name": "456",
            }
        ]);

        if let Err(GiteaError::Parse(issue)) = parse_numbers(val) {
            assert_eq!("Can't parse closed id's".to_owned(), issue)
        } else {
            assert!(false)
        }
    }
}
