use crate::src::announcements::Announcement;
use crate::src::articles::Article;
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{Connection, Result, ToSql};

pub fn establish_connection() -> Result<Connection> {
    Connection::open("./db/database.db")
}

// pub fn create_table() -> Result<()> {
//     let conn = establish_connection()?;

//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS articles (
//             id INTEGER PRIMARY KEY,
//             image TEXT NOT NULL,
//             title TEXT NOT NULL,
//             content TEXT NOT NULL,
//             date TEXT NOT NULL,
//             author TEXT NOT NULL
//         )",
//         [],
//     )?;

//     Ok(())
// }

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

pub fn get_articles(page: i32, page_size: i32) -> Result<(Vec<Article>, i32), rusqlite::Error> {
    let conn = establish_connection()?;

    let offset = (page - 1) * page_size;

    let mut stmt = conn.prepare("SELECT * FROM articles ORDER BY id DESC LIMIT ?1 OFFSET ?2")?;
    let article_iter =
        stmt.query_map(&[&page_size as &dyn ToSql, &offset as &dyn ToSql], |row| {
            let article = Article {
                id: row.get(0)?,
                image: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                date: row.get(4)?,
                author: row.get(5)?,
            };
            Ok(article)
        })?;

    let mut articles = Vec::new();
    for article in article_iter {
        articles.push(article?);
    }

    let total_articles: i32 =
        conn.query_row("SELECT COUNT(*) FROM articles", [], |row| row.get(0))?;

    Ok((articles, total_articles))
}

pub fn get_article(id: i32) -> Result<Article> {
    let conn = establish_connection()?;

    let mut stmt = conn.prepare("SELECT * FROM articles WHERE id = ?1")?;
    let article_iter = stmt.query_map(&[&id.to_string() as &dyn ToSql], |row| {
        let article = Article {
            id: row.get(0)?,
            image: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            date: row.get(4)?,
            author: row.get(5)?,
        };
        Ok(article)
    })?;

    let mut articles = Vec::new();
    for article in article_iter {
        articles.push(article?);
    }
    Ok(articles[0].clone())
}

pub fn add_article(
    image: &str,
    title: &str,
    content: &str,
    date: &str,
    author: &str,
) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "INSERT INTO articles (image, title, content, date, author) VALUES (?1, ?2, ?3, ?4, ?5)",
        &[image, title, content, date, author],
    )?;

    Ok(())
}

pub fn edit_article(
    id: i32,
    image: &str,
    title: &str,
    content: &str,
    date: &str,
    author: &str,
) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "UPDATE articles SET image = ?1, title = ?2, content = ?3, date = ?4, author = ?5 WHERE id = ?6",
        &[image, title, content, date, author, &id.to_string()],
    )?;

    Ok(())
}

pub fn delete_article(id: i32) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute("DELETE FROM articles WHERE id = ?1", &[&id.to_string()])?;

    Ok(())
}

pub fn contact_message(name: &str, email: &str, message: &str, ip_address: &str) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute(
        "INSERT INTO messages (name, email, message, ip_address) VALUES (?1, ?2, ?3, ?4)",
        &[&name, &email, &message, &ip_address],
    )?;

    Ok(())
}

pub fn get_messages() -> Result<Vec<(String, String, String, String)>, rusqlite::Error> {
    let conn = establish_connection()?;

    let mut stmt = conn.prepare("SELECT name, email, message, ip_address FROM messages")?;
    let message_iter = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let email: String = row.get(1)?;
        let message: String = row.get(2)?;
        let ip_address: String = row.get(3)?;
        Ok((name, email, message, ip_address))
    })?;

    let mut messages = Vec::new();
    for message in message_iter {
        messages.push(message?);
    }

    Ok(messages)
}

pub fn authenticate_user(
    username: &str,
    password: &str,
) -> Result<(bool, Option<String>), rusqlite::Error> {
    let conn = establish_connection()?;

    let mut stmt = conn.prepare("SELECT password, name FROM users WHERE username = ?1")?;
    let mut user_iter = stmt.query_map(&[username], |row| {
        let hashed_password: String = row.get(0)?;
        let name: String = row.get(1)?;
        let is_password_match = verify(password, &hashed_password).unwrap_or(false);
        Ok((is_password_match, name))
    })?;

    let user = user_iter
        .filter_map(|result| match result {
            Ok((true, name)) => Some(name),
            _ => None,
        })
        .next();

    Ok((user.is_some(), user))
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

pub fn get_user(username: &str) -> Result<(String, String), rusqlite::Error> {
    let conn = establish_connection()?;

    let mut stmt = conn.prepare("SELECT name,username FROM users WHERE username = ?1")?;
    let mut user_iter = stmt.query_map(&[username], |row| {
        let name: String = row.get(0)?;
        let username: String = row.get(1)?;
        Ok((name, username))
    })?;

    let mut user = user_iter
        .next()
        .unwrap_or(Ok(("".to_string(), "".to_string())))?;

    Ok(user)
}

pub fn add_user(
    name: &str,
    username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = establish_connection()?;
    let hashed_password = hash(password, DEFAULT_COST)?;
    conn.execute(
        "INSERT INTO users (name, username, password) VALUES (?1, ?2, ?3)",
        &[&name, &username, &hashed_password as &str],
    )?;
    Ok(())
}

pub fn delete_user(username: &str) -> Result<()> {
    let conn = establish_connection()?;

    conn.execute("DELETE FROM users WHERE username = ?1", &[&username])?;

    Ok(())
}

pub fn edit_user(
    username: &str,
    name: &str,
    new_username: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if password.is_empty() {
        let conn = establish_connection()?;
        conn.execute(
            "UPDATE users SET name = ?1, username = ?2 WHERE username = ?3",
            &[&name, &new_username, &username],
        )?;
        return Ok(());
    } else {
        let conn = establish_connection()?;
        let hashed_password = hash(password, DEFAULT_COST)?;
        conn.execute(
            "UPDATE users SET name = ?1, username = ?2, password = ?3 WHERE username = ?4",
            &[&name, &new_username, &hashed_password as &str, &username],
        )?;
        return Ok(());
    }
}
