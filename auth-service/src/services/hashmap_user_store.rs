use std::collections::HashMap;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

#[derive(Debug, Default)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let result; 
        if self.get_user(&user.email).await.is_ok() {
            result = Err(UserStoreError::UserAlreadyExists);
        } else {
            self.users.insert(user.email.clone(), user);
            result = Ok(())
        }
        result
    }

    async fn get_user(&self, email: &Email) -> Result<&User, UserStoreError>{
        let result; 
        if let Some(user) = self.users.get(email) {
            result = Ok(user);
        } else {
            result =  Err(UserStoreError::UserNotFound);
        }
        result
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        let result;
        if let Ok(user) = self.get_user(&email).await {
            if user.password.eq(password) {
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
    use crate::domain::{Email, User, Password};
    use crate::services::hashmap_user_store::{HashmapUserStore, UserStoreError};
    use crate::domain::UserStore;

    const TEST_EMAIL: &str = "test@example.com";
    const TEST_PASSWORD: &str = "Asdf1234";

    #[tokio::test]
    async fn test_add_user(){
        let mut test_subject = HashmapUserStore::default();
        let input = User::new(
            Email::parse(&TEST_EMAIL).unwrap(),
            Password::parse(&TEST_PASSWORD).unwrap(), 
            true);
        
        let result = test_subject.add_user(input).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adding_same_user_and_expect_error() {
        let mut test_subject = HashmapUserStore::default();
        let input = User::new(
            Email::parse(&TEST_EMAIL).unwrap(), 
            Password::parse(&TEST_PASSWORD).unwrap(), 
            true);
        let _ = test_subject.add_user(input).await;

        let input2 = User::new(
            Email::parse(&TEST_EMAIL).unwrap(), 
            Password::parse(&TEST_PASSWORD).unwrap(), 
            true);
        let result = test_subject.add_user(input2).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserAlreadyExists)
    }

    #[tokio::test]
    async fn test_get_user(){
        let mut test_subject = HashmapUserStore::default();
        let input = User::new(
            Email::parse(&TEST_EMAIL).unwrap(),
            Password::parse(&TEST_PASSWORD).unwrap(),
            true);

        let _ = test_subject.add_user(input).await;

        let expected_user = User::new(
            Email::parse(&TEST_EMAIL).unwrap(),
            Password::parse(&TEST_PASSWORD).unwrap(),
            true);

        let result = test_subject.get_user(&expected_user.email).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_that_does_not_exist(){
        let test_subject = HashmapUserStore::default();
        let input = User::new(
            Email::parse(&TEST_EMAIL).unwrap(),
            Password::parse(&TEST_PASSWORD).unwrap(),
            true);

        let result = test_subject.get_user(&input.email).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_user(){
        let mut test_subject = HashmapUserStore::default();
        let input = User::new(
            Email::parse(&TEST_EMAIL).unwrap(),
            Password::parse(&TEST_PASSWORD).unwrap(),
            true);
        
        let _ = test_subject.add_user(input).await;

        let expected_input = User::new(
            Email::parse(&TEST_EMAIL).unwrap(),
            Password::parse(&TEST_PASSWORD).unwrap(),
            true);

        let result = test_subject.validate_user(
            &expected_input.email, 
            &expected_input.password
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_with_invalid_password(){
        let mut test_subject = HashmapUserStore::default();
        let input = User::new(
            Email::parse(&TEST_EMAIL).unwrap(),
            Password::parse(&TEST_PASSWORD).unwrap(),
            true);
        
        let _ = test_subject.add_user(input).await;

        let input = User::new(
            Email::parse(&TEST_EMAIL).unwrap(),
            Password::parse(&"asdef1234".to_string()).unwrap(),
            true);
            

        let result = test_subject.validate_user(
            &input.email, 
            &input.password
        ).await;

        assert!(result.is_err());
    }

     #[tokio::test]
    async fn test_validate_user_that_does_not_exist(){
        let test_subject = HashmapUserStore::default();

        let input = User::new(
            Email::parse(&"test@example.com".to_owned()).unwrap(),
            Password::parse(&"Asdef1234".to_owned()).unwrap(),
            true);

        let result = test_subject.validate_user(
            &input.email,
            &input.password
        ).await;

        assert!(result.is_err());
    }
}