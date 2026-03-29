"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const node_cron_1 = __importDefault(require("node-cron"));
const bcrypt_1 = __importDefault(require("bcrypt"));
const jsonwebtoken_1 = __importDefault(require("jsonwebtoken"));
const dotenv_1 = __importDefault(require("dotenv"));
const db_1 = require("./db");
const models_1 = require("./types/models");
const auth_1 = require("./middleware/auth");
dotenv_1.default.config();
const app = (0, express_1.default)();
const port = parseInt(process.env.PORT || '3000', 10);
const host = process.env.SERVER_HOST || '0.0.0.0';
app.listen(port, host, () => {
    console.log(`Server is running at http://${host}:${port}`);
});
app.use(express_1.default.json());
const secretKey = process.env.JWT_SECRET_KEY;
if (!secretKey)
    throw new Error("JWT_SECRET_KEY is not set");
// Create tables
db_1.db.run(`CREATE TABLE IF NOT EXISTS owners (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  lastname TEXT NOT NULL,
  mail TEXT NOT NULL UNIQUE
)`);
db_1.db.run(`CREATE TABLE IF NOT EXISTS books (
  isbn TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  author TEXT NOT NULL,
  owner_id INTEGER NOT NULL REFERENCES owners(id),
  available BOOLEAN NOT NULL DEFAULT 1,
  condition INTEGER NOT NULL,
  borrow_date DATE NULL
)`);
db_1.db.run(`CREATE TABLE IF NOT EXISTS users (
  username TEXT PRIMARY KEY,
  password_hash TEXT NOT NULL,
  user_role INTEGER NOT NULL DEFAULT 0
)`);
db_1.db.run(`CREATE TABLE IF NOT EXISTS tokens (
  token TEXT PRIMARY KEY,
  created_at DATE NOT NULL,
  user TEXT NOT NULL REFERENCES users(username)
)`);
db_1.db.run(`CREATE TABLE IF NOT EXISTS history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  action TEXT NOT NULL,
  occurred_at DATE NOT NULL,
  user TEXT NOT NULL REFERENCES users(username),
  book_isbn TEXT REFERENCES books(isbn),
  target_user TEXT REFERENCES users(username),
  target_owner TEXT REFERENCES owners(id)
)`);
node_cron_1.default.schedule("0 0 * * *", () => {
    db_1.db.run(`DELETE
    FROM tokens
    WHERE created_at <= DATE('now', '-1 days')`, (err) => {
        if (err)
            console.error("Token cleanup failed:", err);
        else
            console.log("Expired tokens cleaned up");
    });
});
// Body contains username and password
app.post("/get-token", async (req, res) => {
    const { username, password } = req.body;
    if (!username || !password)
        return res.status(400).json({ error: "No username or password provided" });
    db_1.db.get(`SELECT *
    FROM users
    WHERE username = ?`, [username], async (err, user) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        if (!user)
            return res.status(400).json({ error: "Inexistant user" });
        if (!await bcrypt_1.default.compare(password, user.password_hash))
            return res.status(403).json({ error: "Wrong password" });
        db_1.db.get(`SELECT *
        FROM tokens
        WHERE user = ?`, [username], (err, token) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            if (token)
                return res.status(200).json({ token: token.token });
            const newToken = jsonwebtoken_1.default.sign({}, secretKey, { expiresIn: '2d' });
            db_1.db.run(`INSERT INTO tokens (token, created_at, user) VALUES (?, ?, ?)`, [newToken, new Date().toISOString(), username], (err) => {
                if (err)
                    return res.status(500).json({ error: "Database error" });
                return res.status(201).json({ token: newToken });
            });
        });
    });
});
// All others api calls need the token in the header
const authMiddleware = (0, auth_1.createAuthMiddleware)(db_1.db);
// No parameter in body
app.get("/check-token", authMiddleware, (req, res) => {
    return res.status(200).json({ message: "Token is valid" });
});
// Body contains username, password and user_level (admin or user)
app.post("/create-user", authMiddleware, async (req, res) => {
    if (req.user?.user_role != 1)
        return res.status(403).json({ error: "You must be admin to execute this task" });
    const username = req.body["username"];
    const role_map = {
        "admin": 1,
        "user": 0,
    };
    const role = role_map[req.body["user_level"]] ?? -1;
    if (!username || !req.body["password"])
        return res.status(400).json({ error: "No username or password provided" });
    if (role === -1)
        return res.status(400).json({ error: "Invalid role, must be 'admin' or 'user'" });
    db_1.db.get(`SELECT *
    FROM users
    WHERE username = ?`, [username], async (err, user) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        if (user)
            return res.status(409).json({ error: "User already exists" });
        const hashed_pwd = await bcrypt_1.default.hash(req.body["password"], 10);
        db_1.db.run(`INSERT INTO users (username, password_hash, user_role) VALUES (?, ?, ?)`, [username, hashed_pwd, role], (err) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            db_1.db.run(`INSERT INTO history (action, occurred_at, user, target_user) VALUES (?, ?, ?, ?)`, [models_1.History_Action.create_user, new Date().toISOString(), req.user?.username, username], (err) => {
                if (err)
                    console.error("Failed to insert history:", err);
                return res.status(201).json({ message: "Successfully added user" });
            });
        });
    });
});
// Body contains username and user_level (admin or user)
app.post("/change-user-level", authMiddleware, async (req, res) => {
    if (req.user?.user_role != 1)
        return res.status(403).json({ error: "You must be admin to execute this task" });
    const username = req.body["username"];
    const role_map = {
        "admin": 1,
        "user": 0,
    };
    const new_role = role_map[req.body["user_level"]] ?? -1;
    if (!username)
        return res.status(400).json({ error: "No username provided" });
    if (new_role === -1)
        return res.status(400).json({ error: "Invalid role, must be 'admin' or 'user'" });
    db_1.db.get(`SELECT *
    FROM users
    WHERE username = ?`, [username], async (err, user) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        if (!user)
            return res.status(409).json({ error: "User doesn't exist" });
        const pre_user_role = user.user_role;
        db_1.db.get(`UPDATE users
        SET user_role = ?
        WHERE username = ?`, [new_role, username], (err) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            if (pre_user_role === new_role)
                return res.status(200).json({ message: "User role is already set to the specified value" });
            const action = new_role === 1 ? models_1.History_Action.promote_user : models_1.History_Action.demote_user;
            db_1.db.run(`INSERT INTO history (action, occurred_at, user, target_user) VALUES (?, ?, ?, ?)`, [action, new Date().toISOString(), req.user?.username, username], (err) => {
                if (err)
                    console.error("Failed to insert history:", err);
                return res.status(201).json({ message: "Successfully updated user role" });
            });
        });
    });
});
// Body contains username
app.post("/delete-user", authMiddleware, async (req, res) => {
    if (req.user?.user_role != 1)
        return res.status(403).json({ error: "You must be admin to execute this task" });
    const username = req.body["username"];
    if (!username)
        return res.status(400).json({ error: "No username provided" });
    db_1.db.get(`SELECT *
    FROM users
    WHERE username = ?`, [username], async (err, user) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        if (!user)
            return res.status(409).json({ error: "User doesn't exist" });
        db_1.db.run(`DELETE FROM users WHERE username = ?`, [username], (err) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            db_1.db.run(`INSERT INTO history (action, occurred_at, user, target_user) VALUES (?, ?, ?, ?)`, [models_1.History_Action.delete_user, new Date().toISOString(), req.user?.username, username], (err) => {
                if (err)
                    console.error("Failed to insert history:", err);
                return res.status(201).json({ message: "Successfully deleted user" });
            });
        });
    });
});
// Body contains name, lastname and mail
app.post("/create-owner", authMiddleware, (req, res) => {
    const { name, lastname, mail } = req.body;
    if (!name || !lastname || !mail)
        return res.status(400).json({ error: "No name, lastname or mail provided" });
    db_1.db.run(`INSERT INTO owners (name, lastname, mail) VALUES (?, ?, ?)`, [name, lastname, mail], (err) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        db_1.db.get(`SELECT *
        FROM owners
        WHERE mail = ? AND name = ? AND lastname = ?`, [mail, name, lastname], (err, owner) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            if (!owner)
                return res.status(500).json({ error: "Failed to retrieve created owner" });
            db_1.db.run(`INSERT INTO history (action, occurred_at, user, target_owner) VALUES (?, ?, ?, ?)`, [models_1.History_Action.create_owner, new Date().toISOString(), req.user?.username, owner.id], (err) => {
                if (err)
                    console.error("Failed to insert history:", err);
                return res.status(201).json({ message: "Successfully added owner" });
            });
        });
    });
});
// Body contains either mail, name or lastname
app.get("/search-owners", authMiddleware, (req, res) => {
    const { mail, name, lastname } = req.body;
    if (!mail && !name && !lastname)
        return res.status(400).json({ error: "No mail, name or lastname provided" });
    let query = `SELECT * FROM owners WHERE 1=1`;
    const params = [];
    if (mail) {
        query += ` AND mail = ?`;
        params.push(mail);
    }
    if (name) {
        query += ` AND name = ?`;
        params.push(name);
    }
    if (lastname) {
        query += ` AND lastname = ?`;
        params.push(lastname);
    }
    db_1.db.all(query, params, (err, owners) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        return res.status(200).json({ owners });
    });
});
// Body contains owner_id
app.post("/delete-owner", authMiddleware, (req, res) => {
    const owner_id = req.body["owner_id"];
    if (!owner_id)
        return res.status(400).json({ error: "No owner_id provided" });
    db_1.db.get(`SELECT *
    FROM owners
    WHERE id = ?`, [owner_id], (err, owner) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        if (!owner)
            return res.status(409).json({ error: "Owner doesn't exist" });
        db_1.db.run(`DELETE FROM owners WHERE id = ?`, [owner_id], (err) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            db_1.db.run(`INSERT INTO history (action, occurred_at, user, target_owner) VALUES (?, ?, ?, ?)`, [models_1.History_Action.delete_owner, new Date().toISOString(), req.user?.username, owner_id], (err) => {
                if (err)
                    console.error("Failed to insert history:", err);
                return res.status(201).json({ message: "Successfully deleted owner" });
            });
        });
    });
});
// Body contains isbn, title, author, owner_id and condition (new, excellent, good, acceptable or bad)
app.post("/add-book", authMiddleware, (req, res) => {
    const { isbn, title, author, owner_id, condition } = req.body;
    const book_map = {
        "new": 3,
        "excellent": 2,
        "good": 1,
        "acceptable": 0,
        "bad": -1,
    };
    const condition_parse = book_map[condition] ?? -2;
    db_1.db.run(`INSERT INTO books (isbn, title, author, owner_id, condition) VALUES (?, ?, ?, ?, ?)`, [isbn, title, author, owner_id, condition_parse], (err) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        db_1.db.run(`INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, ?, ?, ?)`, [models_1.History_Action.add_book, new Date().toISOString(), req.user?.username, isbn], (err) => {
            if (err)
                console.error("Failed to insert history:", err);
            return res.status(201).json({ message: "Successfully added book" });
        });
    });
});
// Body contains either isbn, title, author or owner_id
app.get("/search-books", authMiddleware, (req, res) => {
    const { isbn, title, author, owner_id } = req.body;
    if (!isbn && !title && !author && !owner_id)
        return res.status(400).json({ error: "No isbn, title, author or owner_id provided" });
    let query = `SELECT * FROM books WHERE 1=1`;
    const params = [];
    if (isbn) {
        query += ` AND isbn = ?`;
        params.push(isbn);
    }
    if (title) {
        query += ` AND title = ?`;
        params.push(title);
    }
    if (author) {
        query += ` AND author = ?`;
        params.push(author);
    }
    if (owner_id) {
        query += ` AND owner_id = ?`;
        params.push(owner_id);
    }
    db_1.db.all(query, params, (err, books) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        return res.status(200).json({ books });
    });
});
// Body contains isbn and borrower_mail
app.post("/borrow-book", authMiddleware, (req, res) => {
    const { isbn, borrower_mail } = req.body;
    if (!isbn)
        return res.status(400).json({ error: "No isbn provided" });
    db_1.db.get(`SELECT *
    FROM books
    WHERE isbn = ?`, [isbn], (err, book) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        if (!book)
            return res.status(409).json({ error: "Book doesn't exist" });
        if (!book.available)
            return res.status(409).json({ error: "Book is already borrowed" });
        db_1.db.run(`UPDATE books
        SET available = 0, borrow_date = ?, borrower_mail = ?
        WHERE isbn = ?`, [new Date().toISOString(), borrower_mail, isbn], (err) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            db_1.db.run(`INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, ?, ?, ?)`, [models_1.History_Action.borrow_book, new Date().toISOString(), req.user?.username, isbn], (err) => {
                if (err)
                    console.error("Failed to insert history:", err);
                return res.status(200).json({ message: "Successfully borrowed book" });
            });
        });
    });
});
// Body contains isbn
app.post("/return-book", authMiddleware, (req, res) => {
    const isbn = req.body["isbn"];
    if (!isbn)
        return res.status(400).json({ error: "No isbn provided" });
    db_1.db.get(`SELECT *
    FROM books
    WHERE isbn = ?`, [isbn], (err, book) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        if (!book)
            return res.status(409).json({ error: "Book doesn't exist" });
        if (book.available)
            return res.status(409).json({ error: "Book is not currently borrowed" });
        db_1.db.run(`UPDATE books
        SET available = 1, borrow_date = NULL, borrower_mail = NULL
        WHERE isbn = ?`, [isbn], (err) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            db_1.db.run(`INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, ?, ?, ?)`, [models_1.History_Action.return_book, new Date().toISOString(), req.user?.username, isbn], (err) => {
                if (err)
                    console.error("Failed to insert history:", err);
                return res.status(200).json({ message: "Successfully returned book" });
            });
        });
    });
});
// Body contains isbn
app.post("/delete-book", authMiddleware, (req, res) => {
    const isbn = req.body["isbn"];
    if (!isbn)
        return res.status(400).json({ error: "No isbn provided" });
    db_1.db.get(`SELECT *
    FROM books
    WHERE isbn = ?`, [isbn], (err, book) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        if (!book)
            return res.status(409).json({ error: "Book doesn't exist" });
        db_1.db.run(`DELETE FROM books WHERE isbn = ?`, [isbn], (err) => {
            if (err)
                return res.status(500).json({ error: "Database error" });
            db_1.db.run(`INSERT INTO history (action, occurred_at, user, book_isbn) VALUES (?, ?, ?, ?)`, [models_1.History_Action.delete_book, new Date().toISOString(), req.user?.username, isbn], (err) => {
                if (err)
                    console.error("Failed to insert history:", err);
                return res.status(201).json({ message: "Successfully deleted book" });
            });
        });
    });
});
// No parameter in body
app.get("/late-borrowed-books", authMiddleware, (req, res) => {
    db_1.db.all(`SELECT *
    FROM books
    WHERE available = 0
    AND borrow_date < DATE('now', '-14 days')`, [], (err, books) => {
        if (err)
            return res.status(500).json({ error: "Database error" });
        return res.status(200).json({ late_books: books });
    });
});
//# sourceMappingURL=server.js.map