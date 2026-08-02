import { Database } from "sqlite3";
import cron from "node-cron";

export const scheduleTokenCleanup = (db: Database) => {
  cron.schedule("0 0 * * *", () => {
    db.run(
      `DELETE FROM tokens WHERE datetime(created_at) <= datetime('now', '-1 day')`,
      (err: Error | null) => {
        if (err)
          console.error("Token cleanup failed:", err);
        else
          console.log("Expired tokens cleaned up");
      }
    );
  });
};
