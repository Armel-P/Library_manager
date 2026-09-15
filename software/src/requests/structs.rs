use serde::{Deserialize, Serialize};
use crate::main_app::models::{BookRow, Owner};

#[derive(Serialize)]
pub struct UserBody {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct MessResponse {
    pub message: String,
}

pub type LoginRequest = UserBody;

#[derive(Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

pub type CreateUserRequest = UserBody;
pub type CreateUserResponse = MessResponse;

#[derive(Serialize, Default)]
pub struct BookSearchRequest {
    pub isbn: Option<i64>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub owner_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct BooksResponse {
    pub books: Vec<BookRow>
}

#[derive(Serialize, Default)]
pub struct OwnerSearchRequest {
    pub name: Option<String>,
    pub lastname: Option<String>,
    pub mail: Option<String>,
}

#[derive(Deserialize)]
pub struct OwnersResponse {
    pub owners: Vec<Owner>,
}

#[derive(Deserialize)]
pub struct LateBooksResponse {
    pub late_books: Vec<BookRow>,
}

#[derive(Serialize, Clone)]
pub struct AddBookRequest {
    pub isbn: i64,
    pub title: String,
    pub author: String,
    pub owner_id: i64,
    pub condition: i8,
}

pub type AddBookResponse = MessResponse;

#[derive(Serialize, Clone)]
pub struct AddOwnerRequest {
    pub name: String,
    pub lastname: String,
    pub mail: String,
}

pub type AddOwnerResponse = MessResponse;

#[derive(Serialize, Clone)]
pub struct SampleBookRequest {
    pub begin: i64,
    pub end: i64
}

#[derive(Deserialize)]
pub struct CountResponse {
    pub count: i64,
}

#[derive(serde::Serialize, Clone)]
pub struct BorrowBookRequest {
    pub isbn: i64,
    pub borrower_mail: String,
}

#[derive(serde::Serialize, Clone)]
pub struct ReturnBookRequest {
    pub isbn: i64,
}
