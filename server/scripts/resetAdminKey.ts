#!/usr/bin/env -S npx tsx

import { db } from "../src/db/db";
import { initSchema } from "../src/db/schema";

async function main() {
    initSchema(db);

    db.run(
        `DELETE FROM admin_keys`,
        [], (err) => {
            if (err) {
                console.error("Failed to reset table:", err);
                process.exit(1);
            }
        }
    );

    const { exec } = require('child_process');

    exec('./scripts/initAdminKey.ts', (err: Error | null) => {});
}

main();