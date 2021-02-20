use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION},
    Client,
};
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

use super::{create_comment_string, Api, ApiError};
use crate::todo::Todo;

#[derive(Debug, Error)]
pub enum GiteaError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error("{0}")]
    Parse(String),
}

pub struct Gitea {
    issues_url: String,
    labels: HashMap<String, u64>,
    token: String,
    client: Client,
}

#[async_trait]
impl Api for Gitea {
    async fn closed_ids(&self) -> Result<Vec<u32>, ApiError> {
        let mut page = 1;
        let mut output = Vec::new();

        loop {
            let json = self
                .get_issues(&[("state", "closed"), ("page", &format!("{}", page))])
                .await?;
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

    async fn report_todo(&self, todo: &mut Todo) -> Result<(), ApiError> {
        let mut json: HashMap<&str, Value> = HashMap::new();
        let comment_str = create_comment_string(todo);

        json.insert("title", json!(todo.title.as_str()));
        json.insert("body", json!(&comment_str));

        if let Some(id) = self.labels.get(&todo.keyword) {
            json.insert("labels", json!(&[id]));
        }

        let response = self.post_todo(&json).await?;

        todo.issue_id = Some(parse_issue(response)?);

        Ok(())
    }
}

impl Gitea {
    pub async fn new(
        base_url: &str,
        token: String,
        user: &str,
        repo: &str,
    ) -> Result<Gitea, ApiError> {
        let label_url = format!("{}/repos/{}/{}/labels", base_url, user, repo);
        let client = Client::new();

        Ok(Gitea {
            issues_url: format!("{}/repos/{}/{}/issues", base_url, user, repo),
            labels: get_labels(&client, &label_url, &token).await?,
            token,
            client,
        })
    }

    async fn get_issues<T>(&self, query: &T) -> Result<Value, GiteaError>
    where
        T: Serialize + ?Sized,
    {
        self.client
            .get(&self.issues_url)
            .headers(create_header(&self.token))
            .query(query)
            .send()
            .await?
            .json::<Value>()
            .await
            .map_err(|err| GiteaError::Request(err))
    }

    async fn post_todo<T>(&self, todo: &T) -> Result<Value, GiteaError>
    where
        T: Serialize + ?Sized,
    {
        self.client
            .post(&self.issues_url)
            .headers(create_header(&self.token))
            .json(todo)
            .send()
            .await?
            .json::<Value>()
            .await
            .map_err(|err| GiteaError::Request(err))
    }
}

fn create_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, format!("token {}", token).parse().unwrap());

    headers
}

async fn get_labels(
    client: &Client,
    url: &str,
    token: &str,
) -> Result<HashMap<String, u64>, GiteaError> {
    let mut page = 1;
    let mut out = Vec::new();

    loop {
        let mut json = get_labels_raw(client, url, token, page).await?;

        if let Some(arr) = json.as_array_mut() {
            if arr.is_empty() {
                break;
            } else {
                out.append(arr);
                page += 1
            }
        } else {
            break;
        }
    }

    Ok(parse_labels(out)?)
}

async fn get_labels_raw(
    client: &Client,
    url: &str,
    token: &str,
    page: i32,
) -> Result<Value, GiteaError> {
    client
        .get(url)
        .headers(create_header(token))
        .query(&[("page", page)])
        .send()
        .await?
        .json::<Value>()
        .await
        .map_err(|e| GiteaError::Request(e))
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

fn parse_labels(val: Vec<Value>) -> Result<HashMap<String, u64>, GiteaError> {
    val.into_iter()
        .map(|v| {
            v.as_object()
                .and_then(|o| Some((o.get("name")?, o.get("id")?)))
                .and_then(|t| Some((t.0.as_str()?.to_owned(), t.1.as_u64()?)))
        })
        .collect::<Option<_>>()
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
        let val = vec![
            json!({
                "name": "123",
                "id": 123
            }),
            json!({
                "name": "456",
                "id": 456
            }),
        ];

        if let Ok(map) = parse_labels(val) {
            assert_eq!(&123, map.get("123").unwrap());
            assert_eq!(&456, map.get("456").unwrap())
        } else {
            assert!(false)
        }
    }

    #[test]
    fn parse_labels_fail() {
        let val = vec![
            json!({
                "name": "123",
            }),
            json!({
                "name": "456",
                "id": 456
            }),
        ];

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
