use crate::todo::Todo;

pub trait Api {
    fn next_issue_id(&self) -> u32;

    fn closed_ids(&self) -> Vec<u32>;

    fn report_todo(&self, todo: &Todo);

    fn report_todos(&self, todos: &Vec<Todo>) {
        for todo in todos {
            self.report_todo(todo)
        }
    }
}
