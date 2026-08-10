use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserBody {
    pub username: String,
    pub password: String,
}

pub type LoginRequest = UserBody;

#[derive(Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

pub type CreateUserRequest = UserBody;

#[derive(Deserialize)]
pub struct CreateUserResponse {
    pub message: String,
}
