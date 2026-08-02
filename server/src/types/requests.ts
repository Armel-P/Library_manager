type Credentials = {
    username: string;
    password: string;
};

export type LoginBody = Credentials;
export type CreateUserBody = Credentials; 

export type DeleteUserBody = {
  username: string;
};

export type CreateOwnerBody = {
    name: string;
    lastname: string;
    mail: string;
};

export type SearchOwnerBody = {
    mail?: string;
    name?: string;
    lastname?: string;
};

export type DeleteOwnerBody = {
    owner_id: number;
};

export type CreateBookBody = {
    isbn: number;
    title: string;
    author: string;
    owner_id: number;
    condition: number;
};

export type SearchBookBody = {
    isbn?: number;
    title?: string;
    author?: string;
    owner_id?: number;
};

export type BorrowBookBody = {
    isbn: number;
    borrower_mail: string;
};

type BookID = {
    isbn: number;
};

export type ReturnBookBody = BookID;
export type DeleteBookBody = BookID;


// etc.