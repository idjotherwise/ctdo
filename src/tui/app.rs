use anyhow::Result;
use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Alignment, Rect},
    style::{
        palette::{
            material::{BLUE, GREEN},
            tailwind::SLATE,
        },
        Color, Modifier, Style, Stylize,
    },
    symbols::{self, border},
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget,
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

pub struct App {
    tasks: TaskList,
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
        let tasks = Task::get_tasks(conn)?;
        let task_list = TaskList {
            items: tasks,
            state: ListState::default(),
        };
        Ok(Self {
            tasks: task_list,
            exit: false,
        })
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
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
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.tasks.state.select_previous(),
            KeyCode::Right => self.tasks.state.select_next(),
            KeyCode::Char('h') => self.select_none(),
            KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('k') => self.select_previous(),
            KeyCode::Char('g') => self.select_first(),
            KeyCode::Char('G') => self.select_last(),
            _ => {}
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

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Ctdo list example")
            .bold()
            .centered()
            .render(area, buf);
    }
    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use jk to move, h to unselect, g/G to go top/bottom")
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
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.tasks.state);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![
            " Select ".into(),
            "<w-s>".blue().bold(),
            " Quit ".into(),
            "<q> ".blue().bold(),
        ]));

        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let counter_text = Text::from(vec![
            Line::from(vec![
                "Value: ".into(),
                self.tasks.items.len().to_string().yellow(),
            ]),
            Line::from(vec![
                "Selected: ".into(),
                match self.tasks.state.selected() {
                    Some(u) => u.to_string().yellow(),
                    None => 0.to_string().yellow(),
                },
            ]),
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
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
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        let app = App::new(conn)?;
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 1                    ┃",
            "┃                   Selected: 0                  ┃",
            "┗━━━━━━━━━━━━ Select <w-s> Quit <q> ━━━━━━━━━━━━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(30, 2, 1, 1), counter_style);
        expected.set_style(Rect::new(21, 3, 5, 1), key_style);
        expected.set_style(Rect::new(32, 3, 4, 1), key_style);
        assert_eq!(buf, expected);
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
