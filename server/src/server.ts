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
  borrow_date DATE NULL,
  borrower_mail TEXT NULL
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
  target_user TEXT REFERENCES users(username),
  target_owner TEXT REFERENCES owners(id)
)`);

cron.schedule("0 0 * * *", () => {
  db.run(
    `DELETE
    FROM tokens
    WHERE datetime(created_at) <= datetime('now', '-1 day')`,
    (err) => {
      if (err)
        console.error("Token cleanup failed:", err);
      else
        console.log("Expired tokens cleaned up");
    });
});

// Body contains username and password
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

          const newToken = jwt.sign({}, secretKey, { expiresIn: '2d' });

          db.run(
            `INSERT INTO tokens (token, created_at, user) VALUES (?, datetime('now'), ?)`,
            [newToken, username],
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

// All others api calls need the token in the header
const authMiddleware = createAuthMiddleware(db);

// No parameter in body
app.get("/check-token", authMiddleware, (req: Request, res: Response) => {
  return res.status(200).json({ message: "Token is valid" });
});

// No parameter in body
app.get("/get-user-infos", authMiddleware, (req: Request, res: Response) => {
  return res.status(200).json({ user: req.user });
});

// Body contains username, password and user_level (admin or user)
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
            `INSERT INTO history (action, occurred_at, user, target_user) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.create_user, req.user?.username, username],
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

// Body contains username and user_level (admin or user)
app.post("/change-user-level", authMiddleware, async (req: Request, res: Response) => {
  if (req.user?.user_role != 1)
    return res.status(403).json({ error: "You must be admin to execute this task" });

  const username = req.body["username"];
  const role_map: Record<string, number> = {
    "admin": 1,
    "user": 0,
  };
  const new_role = role_map[req.body["user_level"]] ?? -1;

  if (!username)
    return res.status(400).json({ error: "No username provided" });
  if (new_role === -1)
    return res.status(400).json({ error: "Invalid role, must be 'admin' or 'user'" });

  db.get<User>(
    `SELECT *
    FROM users
    WHERE username = ?`,
    [username],
    async (err, user) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!user)
        return res.status(409).json({ error: "User doesn't exist" });

      const pre_user_role = user.user_role;

      db.get(
        `UPDATE users
        SET user_role = ?
        WHERE username = ?`,
        [new_role, username],
        (err) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          if (pre_user_role === new_role)
            return res.status(200).json({ message: "User role is already set to the specified value" });

          const action = new_role === 1 ? History_Action.promote_user : History_Action.demote_user;

          db.run(
            `INSERT INTO history (action, occurred_at, user, target_user) VALUES (?, datetime('now'), ?, ?)`,
            [action, req.user?.username, username],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);
              return res.status(201).json({ message: "Successfully updated user role" });
            }
          );
        }
      );
    }
  );
});

// Body contains username
app.post("/delete-user", authMiddleware, async (req: Request, res: Response) => {
  if (req.user?.user_role != 1)
    return res.status(403).json({ error: "You must be admin to execute this task" });

  const username = req.body["username"];

  if (!username)
    return res.status(400).json({ error: "No username provided" });

  db.get<User>(
    `SELECT *
    FROM users
    WHERE username = ?`,
    [username],
    async (err, user) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!user)
        return res.status(409).json({ error: "User doesn't exist" });

      db.run(
        `DELETE FROM users WHERE username = ?`,
        [username],
        (err) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, target_user) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.delete_user, req.user?.username, username],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);

              return res.status(201).json({ message: "Successfully deleted user" });
            }
          );
        }
      );
    }
  );
});

// Body contains name, lastname and mail
app.post("/create-owner", authMiddleware, (req: Request, res: Response) => {
  const { name, lastname, mail } = req.body;

  if (!name || !lastname || !mail)
    return res.status(400).json({ error: "No name, lastname or mail provided" });

  db.run(
    `INSERT INTO owners (name, lastname, mail) VALUES (?, ?, ?)`,
    [name, lastname, mail],
    (err) => {
      if (err)
        return res.status(500).json({ error: "Database error" });

      db.get<Owner>(
        `SELECT *
        FROM owners
        WHERE mail = ? AND name = ? AND lastname = ?`,
        [mail, name, lastname],
        (err, owner) => {
          if (err)
            return res.status(500).json({ error: "Database error" });
          if (!owner)
            return res.status(500).json({ error: "Failed to retrieve created owner" });
  
          db.run(
            `INSERT INTO history (action, occurred_at, user, target_owner) VALUES (?, datetime('now')?, ?, ?)`,
            [History_Action.create_owner, req.user?.username, owner.id],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);

              return res.status(201).json({ message: "Successfully added owner" });
            }
          );
        }
      );
    }
  );
});

// Body contains either mail, name or lastname
app.post("/search-owners", authMiddleware, (req: Request, res: Response) => {
  const { mail, name, lastname } = req.body;
  
  if (!mail && !name && !lastname)
    return res.status(400).json({ error: "No mail, name or lastname provided" });

  let query = `SELECT * FROM owners WHERE 1=1`;
  const params: any[] = [];

  if (mail) {
    query += ` AND mail LIKE ?`;
    params.push(`%${mail}%`);
  }
  if (name) {
    query += ` AND name LIKE ?`;
    params.push(`%${name}%`);
  }
  if (lastname) {
    query += ` AND lastname LIKE ?`;
    params.push(`%${lastname}%`);
  }

  db.all<Owner>(query, params, (err, owners) => {
    if (err)
      return res.status(500).json({ error: "Database error" });

    return res.status(200).json({ owners });
  });
});

// Body contains owner_id
app.post("/delete-owner", authMiddleware, (req: Request, res: Response) => {
  const owner_id = req.body["owner_id"];

  if (!owner_id)
    return res.status(400).json({ error: "No owner_id provided" });

  db.get<Owner>(
    `SELECT *
    FROM owners
    WHERE id = ?`,
    [owner_id],
    (err, owner) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!owner)
        return res.status(409).json({ error: "Owner doesn't exist" });

      db.run(
        `DELETE FROM owners WHERE id = ?`,
        [owner_id],
        (err) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, target_owner) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.delete_owner, req.user?.username, owner_id],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);

              return res.status(201).json({ message: "Successfully deleted owner" });
            }
          );
        }
      );
    }
  );
});

// Body contains isbn, title, author, owner_id and condition (new, excellent, good, acceptable or bad)
app.post("/add-book", authMiddleware, (req: Request, res: Response) => {
  const { isbn, title, author, owner_id, condition } = req.body;
  const book_map: Record<string, number> = {
    "new": 3,
    "excellent": 2,
    "good": 1,
    "acceptable": 0,
    "bad": -1,
  };
  const condition_parse = book_map[condition] ?? -2;

  db.run(
    `INSERT INTO books (isbn, title, author, owner_id, condition) VALUES (?, ?, ?, ?, ?)`,
    [isbn, title, author, owner_id, condition_parse],
    (err) => {
      if (err)
        return res.status(500).json({ error: "Database error" });

        db.run(
            `INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.add_book, req.user?.username, isbn],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);
              return res.status(201).json({ message: "Successfully added book" });
            }
          );
        }
    );
});

// Body contains either isbn, title, author or owner_id
app.post("/search-books", authMiddleware, (req: Request, res: Response) => {
  const { isbn, title, author, owner_id } = req.body;

  if (!isbn && !title && !author && !owner_id)
    return res.status(400).json({ error: "No isbn, title, author or owner_id provided" });

  let query = `SELECT * FROM books WHERE 1=1`;
  const params: any[] = [];


  if (isbn) {
    query += ` AND isbn LIKE ?`;
    params.push(`%${isbn}%`);
  }
  if (title) {
    query += ` AND title LIKE ?`;
    params.push(`%${title}%`);
  }
  if (author) {
    query += ` AND author LIKE ?`;
    params.push(`%${author}%`);
  }
  if (owner_id) {
    query += ` AND owner_id LIKE ?`;
    params.push(`%${owner_id}%`);
  }

  db.all<Book>(query, params, (err, books) => {
    if (err)
      return res.status(500).json({ error: "Database error" });

    return res.status(200).json({ books });
  });
});

// Body contains isbn and borrower_mail
app.post("/borrow-book", authMiddleware, (req: Request, res: Response) => {
  const {isbn, borrower_mail} = req.body;

  if (!isbn)
    return res.status(400).json({ error: "No isbn provided" });

  db.get<Book>(
    `SELECT *
    FROM books
    WHERE isbn = ?`,
    [isbn],
    (err, book) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!book)
        return res.status(409).json({ error: "Book doesn't exist" });
      if (!book.available)
        return res.status(409).json({ error: "Book is already borrowed" });
      db.run(
        `UPDATE books
        SET available = 0, borrow_date = datetime('now'), borrower_mail = ?
        WHERE isbn = ?`,
        [borrower_mail, isbn],
        (err) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.borrow_book, req.user?.username, isbn],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);

              return res.status(200).json({ message: "Successfully borrowed book" });
            }
          );
        }
      );
    }
  );
});

// Body contains isbn
app.post("/return-book", authMiddleware, (req: Request, res: Response) => {
  const isbn = req.body["isbn"];

  if (!isbn)
    return res.status(400).json({ error: "No isbn provided" });

  db.get<Book>(
    `SELECT *
    FROM books
    WHERE isbn = ?`,
    [isbn],
    (err, book) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!book)
        return res.status(409).json({ error: "Book doesn't exist" });
      if (book.available)
        return res.status(409).json({ error: "Book is not currently borrowed" });
      db.run(
        `UPDATE books
        SET available = 1, borrow_date = NULL, borrower_mail = NULL
        WHERE isbn = ?`,
        [isbn],
        (err) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.return_book, req.user?.username, isbn],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);

              return res.status(200).json({ message: "Successfully returned book" });
            }
          );
        }
      );
    }
  );
});

// Body contains isbn
app.post("/delete-book", authMiddleware, (req: Request, res: Response) => {
  const isbn = req.body["isbn"];

  if (!isbn)
    return res.status(400).json({ error: "No isbn provided" });

  db.get<Book>(
    `SELECT *
    FROM books
    WHERE isbn = ?`,
    [isbn],
    (err, book) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!book)
        return res.status(409).json({ error: "Book doesn't exist" });

      db.run(
        `DELETE FROM books WHERE isbn = ?`,
        [isbn],
        (err) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.delete_book, req.user?.username, isbn],
            (err) => {
              if (err)
                console.error("Failed to insert history:", err);

              return res.status(201).json({ message: "Successfully deleted book" });
            }
          );
        }
      );
    }
  );
});

// No parameter in body
app.post("/late-borrowed-books", authMiddleware, (req: Request, res: Response) => {
  db.all<Book>(
    `SELECT *
    FROM books
    WHERE available = 0
    AND datetime(borrow_date) < datetime('now', '-14 day')`,
    [],
    (err, books) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      return res.status(200).json({ late_books: books });
    });
});
