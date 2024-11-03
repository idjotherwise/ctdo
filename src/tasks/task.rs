use chrono::{NaiveDateTime, Utc};
use rusqlite::{named_params, params, Connection, Row};

use super::category::Category;

#[derive(Debug, Clone)]
pub struct Task {
    pub title: String,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub completed: Option<bool>,
    pub completed_at: Option<NaiveDateTime>,
    pub category: Category,
    pub id: Option<i64>,
}

impl Task {
    pub fn map(row: &Row) -> Result<Task, rusqlite::Error> {
        Ok(Task {
            title: row.get(0).expect("Could not convert title"),
            description: row.get(1).expect("Could not convert description"),
            created_at: row.get(2).expect("Could not parse created_at"),
            completed_at: None,
            completed: row.get(3).expect("Could not  get task completed status"),
            category: Category {
                name: row.get(4).expect("Could not get category name"),
                color: row.get(5).expect("Could not get category color"),
            },
            id: row.get(6).expect("Could not get task id"),
        })
    }

    pub fn default() -> Self {
        Self {
            title: "".to_string(),
            description: Some("".to_string()),
            created_at: Some(Utc::now().naive_utc()),
            completed: Some(false),
            completed_at: None,
            category: Category::default(),
            id: None,
        }
    }
    pub fn get_tasks(conn: &Connection) -> rusqlite::Result<Vec<Task>> {
        let mut stmt = conn.prepare(
        "select t.title, t.description, t.created_at, t.completed, tt.name, tt.color, t.id from tasks t
              inner join categories tt
              on tt.id = t.category_id;",
    )?;
        let tasks = stmt
            .query_map((), Task::map)?
            .filter_map(Result::ok)
            .collect();

        Ok(tasks)
    }
    pub fn add_task(conn: &Connection, task: &Task) -> rusqlite::Result<()> {
        conn.execute(
            "insert into categories (name, color) values (?1, ?2)",
            (&task.category.name, &task.category.color),
        )?;
        let last_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO tasks (title, description, category_id) VALUES (?1, ?2, ?3)",
            (&task.title, &task.description, &last_id.to_string()),
        )?;
        Ok(())
    }
    pub fn ensure_tables(conn: &Connection) -> rusqlite::Result<()> {
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
        Ok(())
    }
    pub fn get_task(conn: &Connection, id: i64) -> rusqlite::Result<Task> {
        let mut q = conn
            .prepare("select * from tasks where id = :id;")
            .expect("Could not prepare query");
        let task = q.query_row(named_params! {":id": id}, Task::map)?;
        Ok(task)
    }

    pub fn update_task(conn: &Connection, this_task: &Task) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE tasks SET title = ?2, description = ?3 WHERE tasks.id = ?1",
            params![this_task.id, this_task.title, this_task.description],
        )?;
        Ok(())
    }
}
