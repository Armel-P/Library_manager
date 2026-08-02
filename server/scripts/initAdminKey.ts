#!/usr/bin/env -S npx tsx

import bcrypt from "bcrypt";
import crypto from "crypto";
import fs from "fs";
import { db } from "../src/db/db";
import { initSchema } from "../src/db/schema";

async function main() {
    initSchema(db);

    const existing = await new Promise<{ id: number } | undefined>((resolve, reject) => {
        db.get<{ id: number }>(
            `SELECT key_hash FROM admin_keys LIMIT 1`,
            [], (err, id) => {
                if (err)
                    reject(err);
                else
                    resolve(id);
            }
        );
    });

    if (existing) {
        console.error("An admin key already exists. Use resetAdminKey.ts to replace it.");
        process.exit(1);
    }

    const rawKey = crypto.randomBytes(32).toString("base64url");
    const hash = await bcrypt.hash(rawKey, 12);

    db.run(
        `INSERT INTO admin_keys (key_hash) VALUES (?)`,
        [hash], (err) => {
            if (err) {
                console.error("Failed to store admin key:", err);
                process.exit(1);
            }

            fs.writeFileSync("admin.key", rawKey, { mode: 0o600 });
            console.log("Admin key generated and written to ./admin.key");
            process.exit(0);
        }
    );
}

main();