use async_trait::async_trait;
use octocrab::{params, Octocrab};

use super::{create_comment_string, Api, ApiError};
use crate::todo::Todo;

pub struct Github<'a> {
    user: &'a str,
    repo: &'a str,
    client: Octocrab,
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

        let result = builder.send().await?;

        todo.issue_id = Some(result.number as u32);

        Ok(())
    }
}

impl<'a> Github<'a> {
    pub fn new(user: &'a str, repo: &'a str, token: String) -> Result<Self, ApiError> {
        let client = Octocrab::builder().personal_token(token).build()?;

        Ok(Github { user, repo, client })
    }
}
