use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};
use std::collections::HashSet;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};

#[derive(Debug, Default)]
pub struct HashsetBannedTokenStore {
    pub tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: Secret<String>) -> Result<()> {
        if !self.tokens.insert(token.expose_secret().to_owned()) {
            return Err(
                BannedTokenStoreError::UnexpectedError(eyre!("Token already exists")).into(),
            );
        }

        Ok(())
    }

    async fn contains_token(&mut self, token: Secret<String>) -> Result<bool> {
        let flag = self.tokens.contains(token.expose_secret());

        Ok(flag)
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_add_and_check_token() {
        use crate::domain::BannedTokenStore;
        use crate::services::hashset_banned_token_store::HashsetBannedTokenStore;

        let mut store = HashsetBannedTokenStore::default();

        let token = "sample_token";

        // Initially, the token should not be banned
        let is_banned = store
            .contains_token(secrecy::Secret::new(token.to_string()))
            .await
            .unwrap();
        assert!(!is_banned, "Token should not be banned initially");

        // Add the token to the banned list
        store
            .add_token(secrecy::Secret::new(token.to_string()))
            .await
            .unwrap();
        // Now, the token should be banned
        let is_banned = store
            .contains_token(secrecy::Secret::new(token.to_string()))
            .await
            .unwrap();
        assert!(is_banned, "Token should be banned after adding");
    }
}
