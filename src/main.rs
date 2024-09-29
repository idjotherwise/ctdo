use chrono::NaiveDate;
use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal,
};
use rusqlite::Connection;
use std::{
    io::{self, Write},
    str::FromStr,
};
#[derive(Debug)]
struct Task {
    title: String,
    description: Option<String>,
    created_at: Option<NaiveDate>,
    completed: Option<bool>,
    completed_at: Option<NaiveDate>,
    category_id: i64,
}

#[derive(Debug)]
struct Category {
    id: i32,
    name: String,
    color: String,
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let _ = get_tasks();

    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    for y in 0..40 {
        for x in 0..150 {
            if (y == 0 || y == 40 - 2) || (x == 0 || x == 150 - 1) {
                // in this loop we are more efficient by not flushing the buffer.
                queue!(
                    stdout,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent("█".red())
                )?;
            }
        }
    }
    stdout.flush()?;
    Ok(())
}
fn get_tasks() -> rusqlite::Result<()> {
    let conn = Connection::open("todos.db")?;

    conn.execute(
        "
        create table if not exists categories (
            id integer primary key,
            name text not null,
            color text not null
        )
    ",
        (),
    )?;
    conn.execute(
        "
        create table if not exists tasks (
        id integer primary key,
        title text not null,
        description text,
        created_at datetime default current_timestamp,
        completed boolean not null default 0,
        completed_at datetime,
        category_id integer,
        foreign key (category_id) references categories (id) on delete set null
        )
    ",
        (),
    )?;

    let t = Task {
        title: "Buy a thing".to_string(),
        description: Some("Go to somewhere and buy a thing".to_string()),
        created_at: NaiveDate::from_ymd_opt(2024, 1, 1),
        completed: Some(false),
        completed_at: NaiveDate::from_ymd_opt(2024, 1, 1),
        category_id: 1,
    };
    // TODO: insert task into table
    // conn.execute(
    //     "INSERT INTO tasks (name, data) VALUES (?1, ?2)",
    //     (&me.name, &me.data),
    // )?;

    Ok(())
}
