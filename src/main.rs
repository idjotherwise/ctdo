use anyhow::Result;
use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, ListState, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};
use tasks::{task::Task, task_list::TaskList};

mod tasks;

pub struct App {
    tasks: TaskList,
    exit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let tasks = Task::get_tasks()?;
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
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instructions = Title::from(Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
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

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::new()?.run(&mut terminal);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() -> Result<()> {
        let app = App::new()?;
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━ Counter App Tutorial ━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━ Decrement <Left> Increment <Right> Quit <Q> ━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);
        assert_eq!(buf, expected);
        Ok(())
    }

    #[test]
    fn handle_key_event() -> Result<()> {
        let mut app = App::new()?;
        app.handle_key_event(KeyCode::Right.into()).unwrap();
        assert_eq!(app.tasks.state.selected(), Some(1));

        app.handle_key_event(KeyCode::Left.into()).unwrap();
        assert_eq!(app.tasks.state.selected(), Some(0));

        let mut app = App::new()?;
        app.handle_key_event(KeyCode::Char('q').into()).unwrap();
        assert!(app.exit);

        Ok(())
    }
}
