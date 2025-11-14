use std::collections::HashMap;

use crate::domain::{Email, Password, User, UserStore, UserStoreError};

#[derive(Debug, Default)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.get_user(&user.email).await.is_ok() {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        if let Some(user) = self.users.get(email) {
            Ok(user.clone())
        } else {
            Err(UserStoreError::UserNotFound)
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        if let Ok(user) = self.get_user(email).await {
            if user.password.eq(password) {
                Ok(())
            } else {
                Err(UserStoreError::InvalidCredentials)
            }
        } else {
            Err(UserStoreError::UserNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use secrecy::Secret;

    use crate::domain::UserStore;
    use crate::domain::{Email, Password, User};
    use crate::services::hashmap_user_store::{HashmapUserStore, UserStoreError};

    const TEST_EMAIL: &str = "test@example.com";
    const TEST_PASSWORD: &str = "Asdf1234";

    #[tokio::test]
    async fn test_add_user() {
        let mut test_subject = HashmapUserStore::default();
        let input = setup_user();
        let result = test_subject.add_user(input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adding_same_user_and_expect_error() {
        let mut test_subject = HashmapUserStore::default();
        let input = setup_user();

        let _ = test_subject.add_user(input).await;

        let input2 = setup_user();
        let result = test_subject.add_user(input2).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserAlreadyExists)
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut test_subject = HashmapUserStore::default();
        let input = setup_user();

        let _ = test_subject.add_user(input).await;

        let expected_user = setup_user();
        let result = test_subject.get_user(&expected_user.email).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_that_does_not_exist() {
        let test_subject = HashmapUserStore::default();
        let input = setup_user();

        let result = test_subject.get_user(&input.email).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut test_subject = HashmapUserStore::default();
        let input = setup_user();

        let _ = test_subject.add_user(input).await;

        let expected_input = setup_user();
        let result = test_subject
            .validate_user(&expected_input.email, &expected_input.password)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_with_invalid_password() {
        let mut test_subject = HashmapUserStore::default();
        let input = setup_user();

        let _ = test_subject.add_user(input).await;

        let input = User::new(
            Email::parse(Secret::new(TEST_EMAIL.to_string())).unwrap(),
            Password::parse(Secret::new("asdef1234".to_string())).unwrap(),
            true,
        );

        let result = test_subject
            .validate_user(&input.email, &input.password)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_user_that_does_not_exist() {
        let test_subject = HashmapUserStore::default();

        let input = User::new(
            Email::parse(Secret::new("test@example.com".to_string())).unwrap(),
            Password::parse(Secret::new("Asdef1234".to_string())).unwrap(),
            true,
        );

        let result = test_subject
            .validate_user(&input.email, &input.password)
            .await;

        assert!(result.is_err());
    }

    pub fn setup_user() -> User {
        User::new(
            Email::parse(Secret::new(TEST_EMAIL.to_string())).unwrap(),
            Password::parse(Secret::new(TEST_PASSWORD.to_string())).unwrap(),
            true,
        )
    }
}
