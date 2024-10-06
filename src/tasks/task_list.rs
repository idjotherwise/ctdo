use ratatui::widgets::ListState;

use super::task::Task;

#[derive(Debug)]
pub struct TaskList {
    pub items: Vec<Task>,
    pub state: ListState,
}
