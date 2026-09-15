// Owner table
export type Owner = {
    id: number;
    name: string;
    lastname: string;
    mail: string;
};

export enum Book_Condition {new = 3, excellent = 2, good = 1, acceptable = 0, bad = -1};

// Book table
export type Book = {
  isbn: number;
  title: string;
  author: string;
  owner: Owner;
  available: boolean;
  condition: Book_Condition;
  borrow_date: Date;
  borrower_mail?: string;
};

export type BookRow = {
  isbn: number;
  title: string;
  author: string;
  owner_id: number;
  available: number;
  condition: Book_Condition;
  borrow_date: string | null;
  borrower_mail?: string | null;
};

// User table
export type User = {
  username: string;
  password_hash: string;
};

// Token table
export type Token = {
  token: string;
  created_at: Date;
  user: User;
};

export type TokenRow = {
  token: string;
  created_at: string;
  username: string;
};

export enum History_Action {
  borrow_book = "borrow book",
  return_book = "return book",
  add_book = "add book",
  delete_book = "delete book",
  create_owner = "create owner",
  delete_owner = "delete owner",
}

// History table
export type History = {
  action: History_Action;
  occurred_at: Date;
  user: User;
  book?: Book;
  target_user?: User;
  target_owner?: Owner;
};

export type HistoryRow = {
  id: number;
  action: History_Action;
  occurred_at: string;
  user: string;
  book_isbn: number | null;
  target_user: string | null;
  target_owner: number | null;
};
