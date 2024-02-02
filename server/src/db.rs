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

pub fn get_announcements(
    page: i32,
    page_size: i32,
) -> Result<(Vec<Announcement>, i32), rusqlite::Error> {
    let conn = establish_connection()?;

    let offset = (page - 1) * page_size;

    let mut stmt =
        conn.prepare("SELECT * FROM announcements ORDER BY id DESC LIMIT ?1 OFFSET ?2")?;
    let announcement_iter =
        stmt.query_map(&[&page_size as &dyn ToSql, &offset as &dyn ToSql], |row| {
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

    let total_announcements: i32 =
        conn.query_row("SELECT COUNT(*) FROM announcements", [], |row| row.get(0))?;

    Ok((announcements, total_announcements))
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
        &[image, title, content, date, author],
    )?;

    Ok(())
}

pub fn edit_announcement(
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
        &[image, title, content, date, author, &id.to_string()],
    )?;

    Ok(())
}

pub fn delete_announcement(id: i32) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "DELETE FROM announcements WHERE id = ?1",
        &[&id.to_string()],
    )?;

    Ok(())
}

pub fn contact_message(name: &str, email: &str, message: &str) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "INSERT INTO messages (name, email, message) VALUES (?1, ?2, ?3)",
        &[&name, &email, &message],
    )?;

    Ok(())
}

pub fn authenticate_user(username: &str, password: &str) -> Result<bool> {
    let conn = establish_connection()?;

    let mut stmt = conn.prepare("SELECT * FROM users WHERE username = ?1 AND password = ?2")?;
    let user_iter = stmt.query_map(&[username, password], |_| Ok(()))?;

    return Ok(user_iter.count() > 0);
}

pub fn get_users() -> Result<Vec<String>, rusqlite::Error> {
    let conn = establish_connection()?;

    let mut stmt = conn.prepare("SELECT name,username FROM users")?;
    let user_iter = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let username: String = row.get(1)?;
        Ok(format!("{} ({})", name, username))
    })?;

    let mut users = Vec::new();
    for user in user_iter {
        users.push(user?);
    }

    Ok(users)
}

pub fn add_user(name: &str, username: &str, password: &str) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "INSERT INTO users (name, username, password) VALUES (?1, ?2, ?3)",
        &[&name, &username, &password],
    )?;

    Ok(())
}

pub fn delete_user(username: &str) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute("DELETE FROM users WHERE username = ?1", &[&username])?;

    Ok(())
}

pub fn edit_user(username: &str, name: &str, password: &str) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "UPDATE users SET name = ?1, password = ?2 WHERE username = ?3",
        &[&name, &password, &username],
    )?;

    Ok(())
}
