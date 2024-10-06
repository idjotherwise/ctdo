use anyhow::Result;

use rusqlite::Connection;
use tui::App;

mod tasks;
mod tui;

fn main() -> Result<()> {
    let connection = Connection::open("todos.db")?;
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::new(connection)?.run(&mut terminal);
    ratatui::restore();
    app_result
}
