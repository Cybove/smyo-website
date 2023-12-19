use crate::src::main_content::Announcement;
use rusqlite::{Connection, Result, ToSql};

pub fn establish_connection() -> Result<Connection> {
    Connection::open("./db/database.db")
}

pub fn create_table() -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS announcements (
            id INTEGER PRIMARY KEY,
            image TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            date TEXT NOT NULL,
            author TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

pub fn add_announcement(
    image: &str,
    title: &str,
    content: &str,
    date: &str,
    author: &str,
) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "INSERT INTO announcements (image, title, content, date, author) VALUES (?1, ?2, ?3, ?4, ?5)",
        &[&image, &title, &content, &date, &author],
    )?;

    Ok(())
}

pub fn get_announcements() -> Result<Vec<Announcement>> {
    let conn = establish_connection()?;

    let mut stmt = conn.prepare("SELECT * FROM announcements")?;
    let announcement_iter = stmt.query_map([], |row| {
        let announcement = Announcement {
            id: row.get(0)?,
            image: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            date: row.get(4)?,
            author: row.get(5)?,
        };
        Ok(announcement)
    })?;

    let mut announcements = Vec::new();
    for announcement in announcement_iter {
        announcements.push(announcement?);
    }
    Ok(announcements)
}

pub fn get_announcement(id: i32) -> Result<Announcement> {
    let conn = establish_connection()?;

    let mut stmt = conn.prepare("SELECT * FROM announcements WHERE id = ?1")?;
    let announcement_iter = stmt.query_map(&[&id.to_string() as &dyn ToSql], |row| {
        let announcement = Announcement {
            id: row.get(0)?,
            image: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            date: row.get(4)?,
            author: row.get(5)?,
        };
        Ok(announcement)
    })?;

    let mut announcements = Vec::new();
    for announcement in announcement_iter {
        announcements.push(announcement?);
    }
    Ok(announcements[0].clone())
}

pub fn update_announcement(
    id: i32,
    image: &str,
    title: &str,
    content: &str,
    date: &str,
    author: &str,
) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "UPDATE announcements SET image = ?1, title = ?2, content = ?3, date = ?4, author = ?5 WHERE id = ?6",
        &[&image, &title, &content, &date, &author, &(*id.to_string())],
    )?;

    Ok(())
}

pub fn delete_announcement(id: i32) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "DELETE FROM announcements WHERE id = ?1",
        &[&id.to_string() as &dyn ToSql],
    )?;

    Ok(())
}
