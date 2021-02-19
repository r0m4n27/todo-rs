use gitea::GiteaError;
use thiserror::Error;

use crate::todo::Todo;

pub mod gitea;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Gitea(#[from] GiteaError),
}

pub trait Api {
    fn closed_ids(&self) -> Result<Vec<u32>, ApiError>;

    /// Reports the todo and gives it an issue_id
    fn report_todo(&self, todo: &mut Todo) -> Result<(), ApiError>;

    fn report_todos(&self, todos: &mut [Todo]) -> Result<(), ApiError> {
        for todo in todos {
            self.report_todo(todo)?
        }

        Ok(())
    }
}
