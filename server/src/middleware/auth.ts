import { Request, Response, NextFunction } from "express";
import { TokenRow } from "../types/models";
import { Database } from "sqlite3";
import bcrypt from "bcrypt";

export const createAuthMiddleware = (db: Database) => {
  return (req: Request, res: Response, next: NextFunction) => {
    const token = req.header("token");

    if (!token)
      return res.status(401).json({ error: "No token provided" });

    db.get<TokenRow>(
      `SELECT tokens.token, tokens.created_at, users.username
       FROM tokens 
       JOIN users ON tokens.user = users.username
       WHERE tokens.token = ?`,
      [token],
      (err: Error | null, row: TokenRow ) => {
        if (err)
          return res.status(500).json({ error: "Database error" });
        if (!row)
          return res.status(401).json({ error: "Invalid token" });

        req.user = {
          username: row.username,
        };
        next();
      }
    );
  };
};

export const createAdminKeyMiddleware = (db: Database) => {
  return (req: Request, res: Response, next: NextFunction) => {
    const adminKey: string | undefined = req.header("XAdminKey");

    if (!adminKey)
      return res.status(401).json({ error: "No admin key provided" });

    db.get(
      `SELECT key_hash FROM admin_keys LIMIT 1`,
      [],
      async (err: Error, row: { key_hash: string }) => {
          if (err)
            return res.status(500).json({ error: "Database error" });
          if (!row.key_hash)
              return res.status(403).json({ error: "Invalid admin key" });
          
          const isValid = await bcrypt.compare(adminKey, row.key_hash);
          if (!isValid)
            return res.status(403).json({ error: "Invalid admin key" });
          next();
      }
    );
  };
};
