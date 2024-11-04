use anyhow::Result;
use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        palette::{
            material::{BLUE, GREEN},
            tailwind::SLATE,
        },
        Color, Modifier, Style, Stylize,
    },
    symbols::{self},
    text::{Line, Span},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal, Frame,
};
use rusqlite::Connection;

use crate::tasks::{Task, TaskList};

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

enum CurrentScreen {
    List,
    Editing,
    Exiting,
}

enum CurrentlyEditing {
    Title,
    Description,
}

pub struct App {
    tasks: TaskList,
    conn: Connection,
    current_screen: CurrentScreen,
    currently_editing: Option<CurrentlyEditing>,
    exit: bool,
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
impl App {
    pub fn new(conn: Connection) -> Result<Self> {
        Task::ensure_tables(&conn)?;
        let tasks = Task::get_tasks(&conn)?;
        let task_list = TaskList {
            items: tasks,
            state: ListState::default(),
        };
        Ok(Self {
            tasks: task_list,
            conn,
            current_screen: CurrentScreen::List,
            currently_editing: None,
            exit: false,
        })
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) -> Result<()> {
        match self.current_screen {
            CurrentScreen::List => match key_event.code {
                KeyCode::Char('q') => self.current_screen = CurrentScreen::Exiting,
                KeyCode::Left => self.tasks.state.select_previous(),
                KeyCode::Right => self.tasks.state.select_next(),
                KeyCode::Char('h') => self.select_none(),
                KeyCode::Char('j') => self.select_next(),
                KeyCode::Char('k') => self.select_previous(),
                KeyCode::Char('g') => self.select_first(),
                KeyCode::Char('G') => self.select_last(),
                KeyCode::Char('o') => self.insert_task(),
                KeyCode::Char('i') => self.edit_task(),
                KeyCode::Char('d') => self.delete_task(),
                _ => {}
            },
            CurrentScreen::Exiting => match key_event.code {
                KeyCode::Char('y') => self.exit(),
                KeyCode::Char('n') | KeyCode::Char('q') => {
                    self.current_screen = CurrentScreen::List
                }
                _ => {}
            },
            CurrentScreen::Editing => match key_event.code {
                KeyCode::Enter => {
                    self.save_task();
                    self.current_screen = CurrentScreen::List;
                    self.currently_editing = None;
                }
                KeyCode::Tab => match self.currently_editing {
                    Some(CurrentlyEditing::Title) => {
                        self.currently_editing = Some(CurrentlyEditing::Description)
                    }
                    Some(CurrentlyEditing::Description) => {
                        self.currently_editing = Some(CurrentlyEditing::Title)
                    }
                    None => {}
                },
                KeyCode::Char(value) => match self.tasks.state.selected() {
                    Some(i) => {
                        let this_task = &mut self.tasks.items[i];
                        match self.currently_editing {
                            Some(CurrentlyEditing::Title) => this_task.title.push(value),
                            Some(CurrentlyEditing::Description) => {
                                match &mut this_task.description {
                                    Some(d) => d.push(value),
                                    None => this_task.description = Some(value.to_string()),
                                }
                            }
                            None => {}
                        }
                    }
                    None => {}
                },
                KeyCode::Backspace => match self.tasks.state.selected() {
                    Some(i) => {
                        let this_task = &mut self.tasks.items[i];
                        match self.currently_editing {
                            Some(CurrentlyEditing::Title) => this_task.title.pop(),
                            Some(CurrentlyEditing::Description) => {
                                match &mut this_task.description {
                                    Some(d) => d.pop(),
                                    None => None,
                                }
                            }
                            None => None,
                        };
                    }
                    None => panic!("this should not happen brother"),
                },

                _ => {}
            },
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_none(&mut self) {
        self.tasks.state.select(None);
    }

    fn select_next(&mut self) {
        self.tasks.state.select_next();
    }

    fn select_previous(&mut self) {
        self.tasks.state.select_previous();
    }

    fn select_first(&mut self) {
        self.tasks.state.select_first();
    }

    fn select_last(&mut self) {
        self.tasks.state.select_last();
    }
    fn insert_task(&mut self) {
        // This way adds a new empty task then assumes we will update the task later
        //
        let t = Task::default();
        Task::add_task(&self.conn, &t).ok();
        self.tasks.items.push(t);
        self.tasks.state.select_last();
        self.edit_task();
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Ctdo list example")
            .bold()
            .centered()
            .render(area, buf);
    }
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let current_keys_hint = {
            match self.current_screen {
            CurrentScreen::List => {
                Span::styled("(jk) to move, (h) to unselect, (g)/(G) to top/bottom, (o) to add, (i) to edit, (q) quit", Style::default().fg(Color::Red))
            }
            CurrentScreen::Editing=> {
                Span::styled("editing ✍️ (Enter) to save, (Tab) to switch fields", Style::default().fg(Color::Red))
            }
            CurrentScreen::Exiting => {
                Span::styled("Are you sure you want to quit? (y) to confirm, (q) or (n) to cancel", Style::default().fg(Color::Red))
            }
        }
        };

        Paragraph::new(Line::from(current_keys_hint))
            .centered()
            .render(area, buf);
    }
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("List").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .tasks
            .items
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let color = alternate_colors(i);
                ListItem::from(task).bg(color)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(match self.currently_editing {
                Some(CurrentlyEditing::Title) => "✍️",
                _ => ">",
            })
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.tasks.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let info = if let Some(i) = self.tasks.state.selected() {
            match self.tasks.items[i].completed {
                Some(true) => format!(
                    "[x] DONE: {}",
                    self.tasks.items[i].description.as_ref().unwrap()
                ),
                Some(false) => format!(
                    "[ ] TODO: {}",
                    self.tasks.items[i].description.as_ref().unwrap()
                ),
                None => "".to_string(),
            }
        } else {
            "Select an item..".to_string()
        };

        let block = match self.currently_editing {
            Some(CurrentlyEditing::Description) => Block::new()
                .title(
                    Line::raw(match self.currently_editing {
                        Some(CurrentlyEditing::Description) => "Preview ✍️",
                        _ => "Preview",
                    })
                    .centered(),
                )
                .borders(Borders::ALL)
                .border_set(symbols::border::EMPTY)
                .border_style(TODO_HEADER_STYLE)
                .bg(NORMAL_ROW_BG)
                .padding(Padding::horizontal(1)),
            _ => Block::new()
                .title(
                    Line::raw(match self.currently_editing {
                        Some(CurrentlyEditing::Description) => "Preview ✍️",
                        _ => "Preview",
                    })
                    .centered(),
                )
                .borders(Borders::TOP)
                .border_set(symbols::border::EMPTY)
                .border_style(TODO_HEADER_STYLE)
                .bg(NORMAL_ROW_BG)
                .padding(Padding::horizontal(1)),
        };

        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn edit_task(&mut self) {
        if self.tasks.items.len() > 0 {
            self.current_screen = CurrentScreen::Editing;
            self.currently_editing = Some(CurrentlyEditing::Title);
        }
    }
    fn save_task(&self) {
        let this_task = if let Some(i) = self.tasks.state.selected() {
            &mut self.tasks.items[i].clone()
        } else {
            &mut self.tasks.items[0].clone()
        };
        Task::update_task(&self.conn, this_task).expect("Could not update the task");
    }

    fn delete_task(&mut self) {
        let (this_task, idx) = if let Some(i) = self.tasks.state.selected() {
            (&mut self.tasks.items[i].clone(), i)
        } else {
            (&mut self.tasks.items[0].clone(), 0)
        };
        Task::delete_task(&self.conn, this_task).expect("Could not delete the task!");
        self.tasks.items.remove(idx);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        App::render_header(header_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
        self.render_footer(footer_area, buf);
    }
}
impl From<&Task> for ListItem<'_> {
    fn from(value: &Task) -> Self {
        let line = match value.completed {
            Some(true) => Line::styled(format!(" ☐ {}", value.title), TEXT_FG_COLOR),
            Some(false) => Line::styled(format!(" ✓ {}", value.title), COMPLETED_TEXT_FG_COLOR),
            None => Line::styled(format!("? {}", value.title), TEXT_FG_COLOR),
        };
        ListItem::new(line)
    }
}
#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;

    use super::*;
    // use ratatui::style::Style;

    #[test]
    fn render() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        let mut binding = App::new(conn)?;
        let app = binding.borrow_mut();
        let mut buf = Buffer::empty(Rect::new(0, 0, 70, 10));

        app.render(buf.area, &mut buf);
        Ok(())
    }

    #[test]
    fn handle_key_event() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        let mut app = App::new(conn)?;
        app.handle_key_event(KeyCode::Right.into()).unwrap();
        app.handle_key_event(KeyCode::Right.into()).unwrap();
        assert_eq!(app.tasks.state.selected(), Some(1));

        app.handle_key_event(KeyCode::Left.into()).unwrap();
        assert_eq!(app.tasks.state.selected(), Some(0));

        let conn = Connection::open_in_memory()?;
        let mut app = App::new(conn)?;
        app.handle_key_event(KeyCode::Char('q').into()).unwrap();
        assert!(app.exit);

        Ok(())
    }
}
