import { Request, Response, NextFunction } from "express";
import { Database } from "sqlite3";
export declare const createAuthMiddleware: (db: Database) => (req: Request, res: Response, next: NextFunction) => Response<any, Record<string, any>> | undefined;
//# sourceMappingURL=auth.d.ts.map