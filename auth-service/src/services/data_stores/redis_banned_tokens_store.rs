use redis::{Commands, Connection};

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
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        if TOKEN_TTL_SECONDS < 0 {
            return Err(BannedTokenStoreError::UnexpectedError);
        }

        let key = get_key(&token);
        if self
            .conn
            .set_ex(key, true, TOKEN_TTL_SECONDS as u64)
            .unwrap()
        {
            Ok(())
        } else {
            Err(BannedTokenStoreError::UnexpectedError)
        }
    }

    async fn contains_token(&mut self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(&token);

        self.conn
            .get::<_, bool>(key)
            .map(|is_banned| is_banned)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)
    }
}

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
