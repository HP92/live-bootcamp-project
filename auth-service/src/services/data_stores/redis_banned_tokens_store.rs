use color_eyre::eyre::{Context, Result};
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, Secret};

use crate::{
    domain::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

pub struct RedisBannedTokenStore {
    conn: Connection,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "Add Token to REDIS", skip_all)]
    async fn add_token(&mut self, token: Secret<String>) -> Result<()> {
        let token_key = get_key(token.expose_secret());
        let value = true;
        let ttl: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("failed to cast TOKEN_TTL_SECONDS to u64")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        let _: () = self
            .conn
            .set_ex(&token_key, value, ttl)
            .wrap_err("failed to set banned token in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "Check if token is banned in REDIS", skip_all)]
    async fn contains_token(&mut self, token: Secret<String>) -> Result<bool> {
        let token_key = get_key(token.expose_secret());

        let is_banned: bool = self
            .conn
            .exists(&token_key)
            .wrap_err("failed to check if token exists in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(is_banned)
    }
}

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
