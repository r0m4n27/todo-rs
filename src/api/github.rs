use std::collections::HashSet;

use async_trait::async_trait;
use octocrab::{params, Octocrab};

use super::{create_comment_string, Api, ApiError};
use crate::todo::Todo;

pub struct Github<'a> {
    user: &'a str,
    repo: &'a str,
    client: Octocrab,
    labels: HashSet<String>,
}

#[async_trait]
impl<'a> Api for Github<'a> {
    async fn closed_ids(&self) -> Result<Vec<u32>, ApiError> {
        let mut page: u32 = 1;
        let mut output = Vec::new();

        loop {
            let issues = self
                .client
                .issues(self.user, self.repo)
                .list()
                .state(params::State::Closed)
                .per_page(100)
                .page(page)
                .send()
                .await?
                .items;

            if issues.is_empty() {
                break;
            } else {
                let mut numbers = issues.into_iter().map(|i| i.number as u32).collect();

                output.append(&mut numbers);
                page += 1
            }
        }

        Ok(output)
    }

    async fn report_todo(&self, todo: &mut Todo) -> Result<(), ApiError> {
        // Create variable otherwise the IssueHandler is dropped
        let handler = self.client.issues(self.user, self.repo);

        let mut builder = handler.create(&todo.title);

        if !todo.comments.is_empty() {
            builder = builder.body(create_comment_string(&todo))
        }

        if self.labels.contains(&todo.keyword) {
            builder = builder.labels(vec![todo.keyword.clone()])
        }

        let result = builder.send().await?;

        todo.issue_id = Some(result.number as u32);

        Ok(())
    }
}

impl<'a> Github<'a> {
    pub async fn new(user: &'a str, repo: &'a str, token: String) -> Result<Github<'a>, ApiError> {
        let client = Octocrab::builder().personal_token(token).build()?;
        let labels = get_labels(user, repo, &client).await?;

        Ok(Github {
            user,
            repo,
            client,
            labels,
        })
    }
}

async fn get_labels(
    user: &str,
    repo: &str,
    client: &Octocrab,
) -> Result<HashSet<String>, ApiError> {
    let mut page: u32 = 1;
    let mut out = HashSet::new();

    loop {
        let labels = client
            .issues(user, repo)
            .list_labels_for_repo()
            .per_page(100)
            .page(page)
            .send()
            .await?
            .items;

        if !labels.is_empty() {
            page += 1;

            for label in labels {
                out.insert(label.name);
            }
        } else {
            break;
        }
    }

    Ok(out)
}
