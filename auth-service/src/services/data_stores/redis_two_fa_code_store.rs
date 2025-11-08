use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

pub struct RedisTwoFACodeStore {
    conn: Connection,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let two_fa_tuple = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );

        let two_fa_string = serde_json::to_string(&two_fa_tuple);
        if two_fa_string.is_err() {
            return Err(TwoFACodeStoreError::UnexpectedError);
        }

        if TEN_MINUTES_IN_SECONDS < 0 {
            return Err(TwoFACodeStoreError::UnexpectedError);
        }

        let key = get_key(&email);
        if self
            .conn
            .set_ex(key, two_fa_string.unwrap(), TEN_MINUTES_IN_SECONDS as u64)
            .unwrap()
        {
            Ok(())
        } else {
            Err(TwoFACodeStoreError::UnexpectedError)
        }
    }

    async fn remove_code(&mut self, email: Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);
        match self.conn.del::<_, i32>(key) {
            Ok(_) => Ok(()),
            Err(_) => Err(TwoFACodeStoreError::UnexpectedError),
        }
    }

    async fn get_code(
        &mut self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(email);
        let result: Result<String, redis::RedisError> = self.conn.get(key);

        match result {
            Ok(value) => {
                let tuple: TwoFATuple = serde_json::from_str(&value)
                    .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

                Ok((
                    LoginAttemptId::parse(tuple.0).unwrap(),
                    TwoFACode::parse(tuple.1).unwrap(),
                ))
            }
            Err(_) => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
