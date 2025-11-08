use std::collections::HashMap;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

#[derive(Debug, Default)]
pub struct HashmapTwoFACodeStore {
    pub codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: Email) -> Result<(), TwoFACodeStoreError> {
        if self.codes.remove(&email).is_some() {
            Ok(())
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }

    async fn get_code(
        &mut self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        if let Some((login_attempt_id, code)) = self.codes.get(email) {
            Ok((login_attempt_id.clone(), code.clone()))
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};
    use crate::services::hashmap_two_fa_store::HashmapTwoFACodeStore;

    async fn setup_store() -> HashmapTwoFACodeStore {
        HashmapTwoFACodeStore::default()
    }

    #[tokio::test]
    async fn test_add_and_get_code() {
        let mut store = setup_store().await;
        let email = Email::parse("test@example.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::parse("123456".to_string()).unwrap();
        store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await
            .unwrap();
        let (retrieved_login_attempt_id, retrieved_code) = store.get_code(&email).await.unwrap();
        assert_eq!(login_attempt_id, retrieved_login_attempt_id);
        assert_eq!(code, retrieved_code);
    }

    #[tokio::test]
    async fn test_remove_code() {
        let mut store = setup_store().await;
        let email = Email::parse("test@example.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::parse("123456".to_string()).unwrap();
        store
            .add_code(email.clone(), login_attempt_id, code)
            .await
            .unwrap();
        store.remove_code(email.clone()).await.unwrap();
        let result = store.get_code(&email).await;
        assert!(matches!(
            result,
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        ));
    }
}
