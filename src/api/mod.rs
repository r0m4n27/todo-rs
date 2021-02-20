use async_trait::async_trait;
use thiserror::Error;

use crate::todo::Todo;
use gitea::GiteaError;

pub mod gitea;
pub mod github;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Gitea(#[from] GiteaError),

    #[error(transparent)]
    Github(#[from] octocrab::Error),
}

#[async_trait]
pub trait Api: Sync {
    async fn closed_ids(&self) -> Result<Vec<u32>, ApiError>;

    /// Reports the todo and gives it an issue_id
    async fn report_todo(&self, todo: &mut Todo) -> Result<(), ApiError>;

    async fn report_todos(&self, todos: &mut [Todo]) -> Result<(), ApiError> {
        for todo in todos {
            self.report_todo(todo).await?
        }

        Ok(())
    }
}

pub fn create_comment_string(todo: &Todo) -> String {
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
}
