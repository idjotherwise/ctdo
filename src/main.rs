use chrono::NaiveDateTime;
use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal,
};
use rusqlite::{Connection, Row};
use std::{
    io::{self, Write},
    str::FromStr,
};

#[derive(Debug)]
struct Task {
    title: String,
    description: Option<String>,
    created_at: Option<NaiveDateTime>,
    completed: Option<bool>,
    completed_at: Option<NaiveDateTime>,
    category: Category,
}

#[derive(Debug)]
struct Category {
    name: String,
    color: String,
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let tasks: Vec<Task> = get_tasks().unwrap().collect::<Vec<Task>>();

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
    queue!(
        stdout,
        cursor::MoveTo(50, 20),
        style::PrintStyledContent("hi".black())
    )?;
    // stdout.flush()?;
    Ok(())
}
fn get_tasks<'c>() -> rusqlite::Result<impl Iterator<Item = Task>> {
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
        completed_at datetime default null,
        category_id integer,
        foreign key (category_id) references categories(id) on delete set null
        )
    ",
        (),
    )?;
    conn.execute("delete from tasks", ())?;
    let category = Category {
        name: "House".to_string(),
        color: "Black".to_string(),
    };
    conn.execute(
        "insert into categories (name, color) values (?1, ?2)",
        (&category.name, &category.color),
    )?;
    let last_id = conn.last_insert_rowid();

    let t = Task {
        title: "Buy a thing".to_string(),
        description: Some("Go to somewhere and buy a thing".to_string()),
        created_at: None,
        completed_at: None,
        completed: Some(false),
        category,
    };
    // TODO: insert task into table
    conn.execute(
        "INSERT INTO tasks (title, description, category_id) VALUES (?1, ?2, ?3)",
        (&t.title, &t.description, &last_id.to_string()),
    )?;

    let mut stmt = conn.prepare(
        "select t.title, t.description, t.created_at, t.completed, tt.name, tt.color, t.id from tasks t
              inner join categories tt
              on tt.id = t.category_id;",
    )?;
    let tasks = stmt
        .query_map((), |row| {
            Ok(Task {
                title: row.get(0).expect("Could not convert title"),
                description: row.get(1).expect("Could not convert description"),
                created_at: row.get(2).expect("Could not parse created_at"),
                completed_at: None,
                completed: row.get(3)?,
                category: Category {
                    name: row.get(4)?,
                    color: row.get(5)?,
                },
            })
        })
        .into_iter();

    // let tasks = raw_tasks.collect::<Vec<_>>();
    // for task in tasks {
    //     println!("Task: {:?}", task);
    // }

    Ok(tasks)
}
