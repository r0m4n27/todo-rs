use crate::todo::Todo;

pub trait Api {
    fn closed_ids(&self) -> Vec<u32>;

    /// Reports the todo and gives it an issue_id
    fn report_todo(&self, todo: &mut Todo);

    fn report_todos(&self, todos: &mut [Todo]) {
        for todo in todos {
            self.report_todo(todo)
        }
    }
}
