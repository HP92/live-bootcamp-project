use crate::domain::{ Email, Password };

#[derive(Debug)]
pub struct User {
    pub email: Result<Email, String>,
    pub password: Result<Password, String>,
    pub requires_2fa: bool
}

impl User {
    pub fn new(email: Result<Email, String>, password: Result<Password, String>, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa
        }
    }
}