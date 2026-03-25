import express, { Request, Response } from "express";
import cron from "node-cron";
import bcrypt from "bcrypt";
import jwt from 'jsonwebtoken';
import dotenv from 'dotenv';
import { db } from "./db";
import { Owner, Book_Condition, Book, User_Role, User, Token, History_Action, History} from "./types/models";
import { createAuthMiddleware } from "./middleware/auth"

dotenv.config();
const app = express();
const port = parseInt(process.env.PORT || '3000', 10);
const host = process.env.SERVER_HOST || '0.0.0.0';
app.listen(port, host, () => {
  console.log(`Server is running at http://${host}:${port}`);
});
app.use(express.json());

const secretKey = process.env.JWT_SECRET_KEY;
if (!secretKey)
  throw new Error("JWT_SECRET_KEY is not set");

// Create tables
db.run(`CREATE TABLE IF NOT EXISTS owners (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  lastname TEXT NOT NULL,
  mail TEXT NOT NULL UNIQUE
)`);

db.run(`CREATE TABLE IF NOT EXISTS books (
  isbn TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  author TEXT NOT NULL,
  owner_id INTEGER NOT NULL REFERENCES owners(id),
  available BOOLEAN NOT NULL DEFAULT 1,
  condition INTEGER NOT NULL,
  borrow_date DATE NULL
)`);

db.run(`CREATE TABLE IF NOT EXISTS users (
  username TEXT PRIMARY KEY,
  password_hash TEXT NOT NULL,
  user_role INTEGER NOT NULL DEFAULT 0
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
  book_isbn TEXT REFERENCES books(isbn),
  target_user TEXT REFERENCES users(username)
)`);

cron.schedule("0 0 * * *", () => {
  db.run(
    `DELETE
    FROM tokens
    WHERE created_at <= DATE('now', '-7 days')`,
    (err) => {
      if (err)
        console.error("Token cleanup failed:", err);
      else
        console.log("Expired tokens cleaned up");
    });
});

app.post("/get-token", async (req: Request, res: Response) => {
  const { username, password } = req.body;

  if (!username || !password)
    return res.status(400).json({ error: "No username or password provided" });

  db.get<User>(
    `SELECT *
    FROM users
    WHERE username = ?`,
    [username],
    async (err, user) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!user)
        return res.status(400).json({ error: "Inexistant user" });
      if (!await bcrypt.compare(password, user.password_hash))
        return res.status(403).json({ error: "Wrong password" })
      db.get<Token>(
        `SELECT *
        FROM tokens
        WHERE user = ?`,
        [username],
        (err, token) => {
          if (err)
            return res.status(500).json({ error: "Database error" });
          if (token)
            return res.status(200).json({ token: token.token});

          const newToken = jwt.sign({}, secretKey, { expiresIn: '7d' });

          db.run(
            `INSERT INTO tokens (token, created_at, user) VALUES (?, ?, ?)`,
            [newToken, new Date().toISOString(), username],
            (err) => {
              if (err)
                return res.status(500).json({ error: "Database error" });

              return res.status(201).json({ token: newToken });
            }
          );
        }
      );
    }
  );
});

const authMiddleware = createAuthMiddleware(db);

app.get("/check-token", authMiddleware, (req: Request, res: Response) => {
  return res.status(200).json({ message: "Token is valid" });
});

app.post("/create-user", authMiddleware, async (req: Request, res: Response) => {
  if (req.user?.user_role != 1)
    return res.status(403).json({ error: "You must be admin to execute this task" });

  const username = req.body["username"];
  const role_map: Record<string, number> = {
    "admin": 1,
    "user": 0,
  };
  const role = role_map[req.body["user_level"]] ?? -1;

  if (!username || !req.body["password"])
    return res.status(400).json({ error: "No username or password provided" });
  if (role === -1)
    return res.status(400).json({ error: "Invalid role, must be 'admin' or 'user'" });

  db.get(
    `SELECT *
    FROM users
    WHERE username = ?`,
    [username],
    async (err, user) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (user)
        return res.status(409).json({ error: "User already exists" });

      const hashed_pwd = await bcrypt.hash(req.body["password"], 10);

      db.run(
        `INSERT INTO users (username, password_hash, user_role) VALUES (?, ?, ?)`,
        [username, hashed_pwd, role],
        (err) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, target_user) VALUES (?, ?, ?, ?)`,
            [History_Action.create_user, new Date().toISOString(), req.user?.username, username],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);
              return res.status(201).json({ message: "Successfully added user" });
            }
          );
        }
      );
    }
  );
});

app.get("/late-borrowed-books", authMiddleware, (req: Request, res: Response) => {
  db.all(
    `SELECT *
    FROM books
    WHERE available = 0
    AND borrow_date < DATE('now', '-14 days')`,
    [],
    (err, books) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      return res.status(200).json({ late_books: books });
    });
});

// More api calls which will be created base on the software needs
// Sum examples: add a book, add an owner, sum requests base on book name,
// owner name, ect...
