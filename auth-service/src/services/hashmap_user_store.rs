use std::collections::HashMap;

use crate::domain::{User, UserStore, UserStoreError};

pub struct HashmapUserStore {
    pub users: HashMap<String, User>
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    fn new() -> Self {
        HashmapUserStore { users: HashMap::new() }
    }

    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let result; 
        if self.get_user(&user.email).await.is_ok() {
            result = Err(UserStoreError::UserAlreadyExists);
        } else {
            self.users.insert(user.email.to_string(), user);
            result = Ok(())
        }
        result
    }

    async fn get_user(&self, email: &String) -> Result<&User, UserStoreError>{
        let result; 
        if let Some(user) = self.users.get(email) {
            result = Ok(user);
        } else {

            result =  Err(UserStoreError::UserNotFound);
        }
        result
    }

    async fn validate_user(&self, email: &String, password: &String) -> Result<(), UserStoreError> {
        let result;
        if let Ok(user) = self.get_user(email).await {
            if password == &user.password {
                result = Ok(())
            } else {
                result = Err(UserStoreError::InvalidCredentials)
            }
        } else {
            result = Err(UserStoreError::UserNotFound);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::User;
    use crate::services::hashmap_user_store::{HashmapUserStore, UserStoreError};
    use crate::domain::UserStore;

    #[tokio::test]
    async fn test_add_user(){
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);
        
        let result = test_subject.add_user(input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adding_same_user_and_expect_error() {
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);
        let _ = test_subject.add_user(input).await;

        let input2 = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);
        let result = test_subject.add_user(input2).await;

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

        let _ = test_subject.add_user(input).await;
        
        let user_email = "test@example.com".to_owned();
        let result = test_subject.get_user(&user_email).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_that_does_not_exist(){
        let test_subject = HashmapUserStore::new();
        let user_email = "test@example.com".to_owned();
        let result = test_subject.get_user(&user_email).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_user(){
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Asdef1234".to_owned(), 
            true);
        
        let _ = test_subject.add_user(input).await;

        let user_email = "test@example.com".to_owned();
        let user_password = "Asdef1234".to_owned();
        let result = test_subject.validate_user(
            &user_email, 
            &user_password
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_with_invalid_password(){
        let mut test_subject = HashmapUserStore::new();
        let input = User::new(
            "test@example.com".to_owned(), 
            "Adsdf1234".to_owned(), 
            true);
        
        let _ = test_subject.add_user(input).await;

        let user_email = "test@example.com".to_owned();
        let invalid_password = "asdef1234".to_owned();
        let result = test_subject.validate_user(
            &user_email, 
            &invalid_password
        ).await;

        assert!(result.is_err());
    }

     #[tokio::test]
    async fn test_validate_user_that_does_not_exist(){
        let test_subject = HashmapUserStore::new();
        let user_email = "test@example.com".to_owned();
        let user_password = "Asdef1234".to_owned();
        let result = test_subject.validate_user(&user_email, &user_password).await;

        assert!(result.is_err());
    }
}