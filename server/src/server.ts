import express, { Request, Response } from "express";
import cron from "node-cron";
import bcrypt from "bcrypt";
import jwt from 'jsonwebtoken';
import dotenv from 'dotenv';
import { db } from "./db/db";
import { initSchema } from "./db/schema";
import { Owner, BookRow, User, TokenRow, History_Action} from "./types/models";
import { LoginBody,
  CreateUserBody, DeleteUserBody,
  CreateOwnerBody, SearchOwnerBody, DeleteOwnerBody,
  CreateBookBody, SearchBookBody, BorrowBookBody, ReturnBookBody, DeleteBookBody, 
  } from "./types/requests"
import { createAuthMiddleware, createAdminKeyMiddleware } from "./middleware/auth"
import { scheduleTokenCleanup } from "./jobs/cleanupTokens";

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

initSchema(db);

scheduleTokenCleanup(db);

app.post("/token/get", async (req: Request<{}, any, LoginBody>, res: Response) => {
  const { username, password } = req.body;

  if (!username || !password)
    return res.status(400).json({ error: "No username or password provided" });

  db.get<User>(
    `SELECT *
    FROM users
    WHERE username = ?`,
    [username],
    async (err: Error | null, user: User) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!user)
        return res.status(400).json({ error: "Inexistant user" });
      if (!await bcrypt.compare(password, user.password_hash))
        return res.status(403).json({ error: "Wrong password" })
      db.get<TokenRow>(
        `SELECT *
        FROM tokens
        WHERE user = ?`,
        [username],
        (err: Error | null, token: TokenRow) => {
          if (err)
            return res.status(500).json({ error: "Database error" });
          if (token)
            return res.status(200).json({ token: token.token});

          const newToken = jwt.sign({}, secretKey, { expiresIn: '2d' });

          db.run(
            `INSERT INTO tokens (token, created_at, user) VALUES (?, datetime('now'), ?)`,
            [newToken, username],
            (err: Error) => {
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

// Middlewares initialization
const authMiddleware = createAuthMiddleware(db);
const adminKeyMiddleware = createAdminKeyMiddleware(db);

app.post("/token/check", authMiddleware, (req: Request, res: Response) => {
  return res.status(200).json({ message: "Token is valid" });
});

app.post("/user/get", authMiddleware, (req: Request, res: Response) => {
  return res.status(200).json({ user: req.user });
});

app.post("/user/create", adminKeyMiddleware, async (req: Request<{}, any, CreateUserBody>, res: Response) => {
  const username = req.body["username"];
  const password = req.body["password"];

  if (!username || !password)
    return res.status(400).json({ error: "No username or password provided" });

  db.get<User>(
    `SELECT *
    FROM users
    WHERE username = (?)`,
    [username],
    async (err: Error | null, user: User) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (user)
        return res.status(409).json({ error: "User already exists" });

      const hashed_pwd = await bcrypt.hash(password, 10);

      db.run(
        `INSERT INTO users (username, password_hash) VALUES (?, ?)`,
        [username, hashed_pwd],
        (err: Error) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          return res.status(201).json({message: "User created successfully"});
        }
      );
    }
  );
});

app.post("/user/delete", adminKeyMiddleware, async (req: Request<{}, any, DeleteUserBody>, res: Response) => {
  const username = req.body["username"];

  if (!username)
    return res.status(400).json({ error: "No username provided" });

  db.get<User>(
    `SELECT *
    FROM users
    WHERE username = ?`,
    [username],
    async (err: Error | null, user: User) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!user)
        return res.status(409).json({ error: "User doesn't exist" });

      db.run(
        `DELETE FROM users WHERE username = ?`,
        [username],
        (err: Error) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          return res.status(200).json({message: "User successfully deleted"});
        }
      );
    }
  );
});

app.post("/owner/create", authMiddleware, (req: Request<{}, any, CreateOwnerBody>, res: Response) => {
  const { name, lastname, mail } = req.body;

  if (!name || !lastname || !mail)
    return res.status(400).json({ error: "No name, lastname or mail provided" });

  db.run(
    `INSERT INTO owners (name, lastname, mail) VALUES (?, ?, ?)`,
    [name, lastname, mail],
    (err: Error) => {
      if (err)
        return res.status(500).json({ error: "Database error" });

      db.get<Owner>(
        `SELECT *
        FROM owners
        WHERE mail = ? AND name = ? AND lastname = ?`,
        [mail, name, lastname],
        (err: Error | null, owner: Owner) => {
          if (err)
            return res.status(500).json({ error: "Database error" });
          if (!owner)
            return res.status(500).json({ error: "Failed to retrieve created owner" });
  
          db.run(
            `INSERT INTO history (action, occurred_at, user, target_owner) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.create_owner, req.user?.username, owner.id],
            (err: Error) => {
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

app.post("/owner/search", authMiddleware, (req: Request<{}, any, SearchOwnerBody>, res: Response) => {
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

  db.all<Owner>(query, params, (err: Error | null, owners: Owner[]) => {
    if (err)
      return res.status(500).json({ error: "Database error" });

    return res.status(200).json({ owners });
  });
});

app.post("/owner/delete", authMiddleware, (req: Request<{}, any, DeleteOwnerBody>, res: Response) => {
  const owner_id = req.body["owner_id"];

  if (!owner_id)
    return res.status(400).json({ error: "No owner_id provided" });

  db.get<Owner>(
    `SELECT *
    FROM owners
    WHERE id = ?`,
    [owner_id],
    (err: Error | null, owner: Owner) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!owner)
        return res.status(409).json({ error: "Owner doesn't exist" });

      db.run(
        `DELETE FROM owners WHERE id = ?`,
        [owner_id],
        (err: Error) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, target_owner) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.delete_owner, req.user?.username, owner_id],
            (err: Error) => {
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

app.post("/book/create", authMiddleware, (req: Request<{}, any, CreateBookBody>, res: Response) => {
  const { isbn, title, author, owner_id, condition } = req.body;

  db.run(
    `INSERT INTO books (isbn, title, author, owner_id, condition) VALUES (?, ?, ?, ?, ?)`,
    [isbn, title, author, owner_id, condition],
    (err: Error) => {
      if (err)
        return res.status(500).json({ error: "Database error" });

        db.run(
            `INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.add_book, req.user?.username, isbn],
            (err: Error) => {
              if (err)
                console.error("Failed to insert history:", err);
              return res.status(201).json({ message: "Successfully added book" });
            }
          );
        }
    );
});

app.post("/book/search", authMiddleware, (req: Request<{}, any, SearchBookBody>, res: Response) => {
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

  db.all<BookRow>(query, params, (err: Error | null, books: BookRow[]) => {
    if (err)
      return res.status(500).json({ error: "Database error" });

    return res.status(200).json({ books });
  });
});

app.post("/book/borrow", authMiddleware, (req: Request<{}, any, BorrowBookBody>, res: Response) => {
  const {isbn, borrower_mail} = req.body;

  if (!isbn)
    return res.status(400).json({ error: "No isbn provided" });

  db.get<BookRow>(
    `SELECT *
    FROM books
    WHERE isbn = ?`,
    [isbn],
    (err: Error | null, book: BookRow) => {
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
        (err: Error) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.borrow_book, req.user?.username, isbn],
            (err: Error) => {
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

app.post("/book/return", authMiddleware, (req: Request<{}, any, ReturnBookBody>, res: Response) => {
  const isbn = req.body["isbn"];

  if (!isbn)
    return res.status(400).json({ error: "No isbn provided" });

  db.get<BookRow>(
    `SELECT *
    FROM books
    WHERE isbn = ?`,
    [isbn],
    (err: Error | null, book: BookRow) => {
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
        (err: Error) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.return_book, req.user?.username, isbn],
            (err: Error) => {
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

app.post("/book/delete", authMiddleware, (req: Request<{}, any, DeleteBookBody>, res: Response) => {
  const isbn = req.body["isbn"];

  if (!isbn)
    return res.status(400).json({ error: "No isbn provided" });

  db.get<BookRow>(
    `SELECT *
    FROM books
    WHERE isbn = ?`,
    [isbn],
    (err: Error | null, book: BookRow) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      if (!book)
        return res.status(409).json({ error: "Book doesn't exist" });

      db.run(
        `DELETE FROM books WHERE isbn = ?`,
        [isbn],
        (err: Error) => {
          if (err)
            return res.status(500).json({ error: "Database error" });

          db.run(
            `INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, datetime('now'), ?, ?)`,
            [History_Action.delete_book, req.user?.username, isbn],
            (err: Error) => {
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

app.post("/book/late-borrowed", authMiddleware, (req: Request, res: Response) => {
  db.all<BookRow>(
    `SELECT *
    FROM books
    WHERE available = 0
    AND datetime(borrow_date) < datetime('now', '-14 day')`,
    [],
    (err: Error | null, books: BookRow[]) => {
      if (err)
        return res.status(500).json({ error: "Database error" });
      return res.status(200).json({ late_books: books });
    });
});
