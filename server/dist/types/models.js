"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.History_Action = exports.User_Role = exports.Book_Condition = void 0;
var Book_Condition;
(function (Book_Condition) {
    Book_Condition[Book_Condition["new"] = 3] = "new";
    Book_Condition[Book_Condition["excellent"] = 2] = "excellent";
    Book_Condition[Book_Condition["good"] = 1] = "good";
    Book_Condition[Book_Condition["acceptable"] = 0] = "acceptable";
    Book_Condition[Book_Condition["bad"] = -1] = "bad";
})(Book_Condition || (exports.Book_Condition = Book_Condition = {}));
;
var User_Role;
(function (User_Role) {
    User_Role[User_Role["admin"] = 1] = "admin";
    User_Role[User_Role["user"] = 0] = "user";
})(User_Role || (exports.User_Role = User_Role = {}));
var History_Action;
(function (History_Action) {
    History_Action["borrow_book"] = "borrow book";
    History_Action["return_book"] = "return book";
    History_Action["add_book"] = "add book";
    History_Action["delete_book"] = "delete book";
    History_Action["create_owner"] = "create owner";
    History_Action["delete_owner"] = "delete owner";
    History_Action["create_user"] = "create user";
    History_Action["delete_user"] = "delete user";
    History_Action["promote_user"] = "promote user";
    History_Action["demote_user"] = "demote user";
})(History_Action || (exports.History_Action = History_Action = {}));
//# sourceMappingURL=models.js.map