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
  isbn: string;
  title: string;
  author: string;
  owner: Owner;
  available: boolean;
  condition: Book_Condition;
  borrow_date: Date;
  borrower_mail?: string;
};

export enum User_Role {admin = 1, user = 0}

// User table
export type User = {
  username: string; //Use as ID, can't be both times the same username
  password_hash: string;
  user_role: User_Role;
};

// Token table
export type Token = {
  token: string;
  created_at: Date;
  user: User;
};

export enum History_Action {
  borrow_book = "borrow book",
  return_book = "return book",
  add_book = "add book",
  delete_book = "delete book",
  create_owner = "create owner",
  delete_owner = "delete owner",
  create_user = "create user",
  delete_user = "delete user",
  promote_user = "promote user",
  demote_user = "demote user",
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
