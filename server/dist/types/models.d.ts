export type Owner = {
    id: number;
    name: string;
    lastname: string;
    mail: string;
};
export declare enum Book_Condition {
    new = 3,
    excellent = 2,
    good = 1,
    acceptable = 0,
    bad = -1
}
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
export declare enum User_Role {
    admin = 1,
    user = 0
}
export type User = {
    username: string;
    password_hash: string;
    user_role: User_Role;
};
export type Token = {
    token: string;
    created_at: Date;
    user: User;
};
export declare enum History_Action {
    borrow_book = "borrow book",
    return_book = "return book",
    add_book = "add book",
    delete_book = "delete book",
    create_owner = "create owner",
    delete_owner = "delete owner",
    create_user = "create user",
    delete_user = "delete user",
    promote_user = "promote user",
    demote_user = "demote user"
}
export type History = {
    action: History_Action;
    occurred_at: Date;
    user: User;
    book?: Book;
    target_user?: User;
    target_owner?: Owner;
};
//# sourceMappingURL=models.d.ts.map