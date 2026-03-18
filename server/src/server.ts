import express, { Request, Response } from "express";
import sqlite3 from "sqlite3";

const app = express();
app.use(express.json());

// Owner type
type Owner = {
    id: number;
    name: string;
    lastname: string;
    phone_number: number;
    mail: string;
};

enum Book_condition {};

// Book type
type Book = {
  id: string;
  title: string;
  author: string;
  owner: Owner;
  available: boolean;
  condition: Book_condition;
  remaining_days: number; // or datetime format of expiration date
};

// Connect DB
const db = new sqlite3.Database("./library.db");

// Create table
db.run(''); // fill

// api calls
