use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError
}

pub struct HashmapUserStore {
    pub users: HashMap<String, User>
}

impl HashmapUserStore {
    pub fn new() -> Self {
        HashmapUserStore { users: HashMap::new() }
    }

    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if let Ok(_) = self.get_user(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        } else {
            self.users.insert(user.email.to_string(), user);
            Ok(())
        }
    }

    pub fn get_user(&self, email: &String) -> Result<&User, UserStoreError>{
        if let Some(user) = self.users.get(email) {
            return Ok(user);
        } else {
            return Err(UserStoreError::UserNotFound);
        }
    }

    pub fn validate_user(&self, email: &String, password: &String) -> Result<(), UserStoreError> {
        if let Ok(user) = self.get_user(email) {
            if password == &user.password {
                Ok(())
            } else {
                return Err(UserStoreError::InvalidCredentials)
            }
        } else {
            return Err(UserStoreError::UserNotFound);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain::User, services::{HashmapUserStore, UserStoreError}};

    #[tokio::test]
    async fn test_add_user(){
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);
        
        let result = test_subject.add_user(input);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adding_same_user_and_expect_error() {
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);
        
        let _ = test_subject.add_user(input);

        let input2 = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);
        let result = test_subject.add_user(input2);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserAlreadyExists)
    }

    #[tokio::test]
    async fn test_get_user(){
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);

        let _ = test_subject.add_user(input);
        
        let user_email = "test@example.com".to_owned();
        let result = test_subject.get_user(&user_email);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_that_does_not_exist(){
        let test_subject = HashmapUserStore::new();
        let user_email = "test@example.com".to_owned();
        let result = test_subject.get_user(&user_email);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_user(){
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Asdef1234".to_owned(), 
            true);
        
        let _ = test_subject.add_user(input);

        let user_email = "test@example.com".to_owned();
        let user_password = "Asdef1234".to_owned();
        let result = test_subject.validate_user(
            &user_email, 
            &user_password
        );

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_with_invalid_password(){
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);
        
        let _ = test_subject.add_user(input);

        let user_email = "test@example.com".to_owned();
        let invalid_password = "asdef1234".to_owned();
        let result = test_subject.validate_user(
            &user_email, 
            &invalid_password
        );

        assert!(result.is_err());
    }

     #[tokio::test]
    async fn test_validate_user_that_does_not_exist(){
        let test_subject = HashmapUserStore::new();
        let user_email = "test@example.com".to_owned();
        let user_password = "Asdef1234".to_owned();
        let result = test_subject.validate_user(&user_email, &user_password);

        assert!(result.is_err());
    }
}