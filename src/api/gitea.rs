use std::collections::HashMap;

use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION},
};
use serde::Serialize;
use serde_json::Value;

use crate::todo::Todo;

use super::{Api, ApiError};

pub struct Gitea<'a> {
    issues_url: String,
    token: &'a str,
    client: Client,
}

impl<'a> Api for Gitea<'a> {
    fn closed_ids(&self) -> Result<Vec<u32>, ApiError> {
        let mut page = 1;
        let mut output = Vec::new();

        loop {
            let json = self.get_issues(&[("state", "closed"), ("page", &format!("{}", page))])?;
            let arr = json.as_array().unwrap();

            if !arr.is_empty() {
                page += 1;

                let mut numbers: Vec<_> = arr
                    .into_iter()
                    .map(|v| {
                        {
                            v.as_object()
                                .and_then(|o| o.get("number"))
                                .and_then(|v| v.as_u64())
                                .and_then(|u| Some(u as u32))
                        }
                        .unwrap()
                    })
                    .collect();

                output.append(&mut numbers)
            } else {
                break;
            }
        }

        Ok(output)
    }

    fn report_todo(&self, todo: &mut Todo) -> Result<(), ApiError> {
        let mut json = HashMap::new();
        let mut comment_str = String::new();

        for comment in &todo.comments {
            if comment == "" {
                comment_str.push_str("\n")
            } else {
                comment_str.push_str(comment)
            }
        }

        json.insert("title", todo.title.as_str());
        json.insert("body", &comment_str);

        let response = self.post_todo(&json);

        let issue = response?
            .as_object()
            .and_then(|o| o.get("number"))
            .and_then(|v| v.as_u64())
            .and_then(|u| Some(u as u32))
            .unwrap();

        todo.issue_id = Some(issue);

        Ok(())
    }
}

impl<'a> Gitea<'a> {
    pub fn new(base_url: &str, token: &'a str, user: &str, repo: &str) -> Self {
        Gitea {
            issues_url: format!("{}/repos/{}/{}/issues", base_url, user, repo),
            token,
            client: Client::new(),
        }
    }

    fn create_header(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            format!("token {}", self.token).parse().unwrap(),
        );

        headers
    }

    fn get_issues<T>(&self, query: &T) -> Result<Value, ApiError>
    where
        T: Serialize + ?Sized,
    {
        self.client
            .get(&self.issues_url)
            .headers(self.create_header())
            .query(query)
            .send()?
            .json::<Value>()
            .map_err(|err| ApiError::Request(err))
    }

    fn post_todo<T>(&self, todo: &T) -> Result<Value, ApiError>
    where
        T: Serialize + ?Sized,
    {
        self.client
            .post(&self.issues_url)
            .headers(self.create_header())
            .json(todo)
            .send()?
            .json::<Value>()
            .map_err(|err| ApiError::Request(err))
    }
}
