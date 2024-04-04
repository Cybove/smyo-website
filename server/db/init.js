const sqlite3 = require("sqlite3").verbose();

let db = new sqlite3.Database("./server/db/database.db", (err) => {
  if (err) {
    return console.error(err.message);
  }
  console.log("Connected to the SQlite database.");
});

db.serialize(() => {
  db.run(`CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            name TEXT NOT NULL
          )`);

  db.run(`CREATE TABLE IF NOT EXISTS announcements (
            id INTEGER PRIMARY KEY,
            image TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            date TEXT NOT NULL,
            author TEXT NOT NULL
          )`);

  db.run(`CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY,
            image TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            date TEXT NOT NULL,
            author TEXT NOT NULL
          )`);

  db.run(`CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            name TEXT,
            email TEXT,
            message TEXT,
            ip_address TEXT
          )`);

  db.run(
    `INSERT INTO users (username, password, name) VALUES ('root', '$2a$12$emHZ1nzkcNjDE/fKV5Ali.xX8TyU8gMRRKH4j35QIrVz5Eozd1.Fa', 'root')`
  );
});

db.close((err) => {
  if (err) {
    return console.error(err.message);
  }
  console.log("Close the database connection.");
});
