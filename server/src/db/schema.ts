import { Database } from "sqlite3";

export const initSchema = (db: Database) => {
  db.run(`CREATE TABLE IF NOT EXISTS owners (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    lastname TEXT NOT NULL,
    mail TEXT NOT NULL UNIQUE
  )`);

  db.run(`CREATE TABLE IF NOT EXISTS books (
    isbn INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    owner_id INTEGER NOT NULL REFERENCES owners(id),
    available BOOLEAN NOT NULL DEFAULT 1,
    condition INTEGER NOT NULL,
    borrow_date DATE NULL,
    borrower_mail TEXT NULL
  )`);

  db.run(`CREATE TABLE IF NOT EXISTS users (
    username TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL
  )`);

  db.run(`CREATE TABLE IF NOT EXISTS tokens (
    token TEXT PRIMARY KEY,
    created_at DATE NOT NULL,
    user TEXT NOT NULL REFERENCES users(username)
  )`);

  db.run(`CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action TEXT NOT NULL,
    occurred_at DATE NOT NULL,
    user TEXT NOT NULL REFERENCES users(username),
    book_isbn INTEGER REFERENCES books(isbn),
    target_user TEXT REFERENCES users(username),
    target_owner INTEGER REFERENCES owners(id)
  )`);

  db.run(`CREATE TABLE IF NOT EXISTS admin_keys (
    key_hash TEXT NOT NULL
  )`);
};