import { Request, Response, NextFunction } from "express";
import { Database } from "sqlite3";

type TokenRow = {
  token: string;
  created_at: string;
  username: string;
  user_role: number;
};

export const createAuthMiddleware = (db: Database) => {
  return (req: Request, res: Response, next: NextFunction) => {
    const token = req.headers["authorization"]?.split(" ")[1];

    if (!token) {
      return res.status(401).json({ error: "No token provided" });
    }

    db.get<TokenRow>(
      `SELECT tokens.token, tokens.created_at, users.username, users.user_role
       FROM tokens 
       JOIN users ON tokens.user = users.username
       WHERE tokens.token = ?`,
      [token],
      (err, row) => {
        if (err)
          return res.status(500).json({ error: "Database error" });
        if (!row)
          return res.status(401).json({ error: "Invalid token" });

        req.user = { username: row.username, user_role: row.user_role };
        next();
      }
    );
  };
};
