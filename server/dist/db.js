"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.db = void 0;
const sqlite3_1 = __importDefault(require("sqlite3"));
exports.db = new sqlite3_1.default.Database("./library.db", (err) => {
    if (err) {
        console.error("Failed to connect to database:", err);
        process.exit(1);
    }
    console.log("Connected to database");
});
//# sourceMappingURL=db.js.map