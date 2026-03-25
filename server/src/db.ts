import sqlite3 from "sqlite3";

export const db = new sqlite3.Database("./library.db", (err) => {
  if (err) {
    console.error("Failed to connect to database:", err);
    process.exit(1);
  }
  console.log("Connected to database");
});
