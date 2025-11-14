use color_eyre::eyre::{Context, Result};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct LoginAttemptId(Secret<String>);

impl LoginAttemptId {
    pub fn parse(id: Secret<String>) -> Result<Self> {
        // Use the `parse_str` function from the `uuid` crate to ensure `id` is a valid UUID
        let parsed_id = uuid::Uuid::parse_str(id.expose_secret())
            .wrap_err("Login Attempt ID doesn't match with UUID format")?;

        Ok(Self(Secret::new(parsed_id.to_string())))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        // Use the `uuid` crate to generate a random version 4 UUID
        let uuid = Secret::new(Uuid::new_v4().to_string());
        Self(uuid)
    }
}

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        // We can use the expose_secret method to expose the secret in a
        // controlled manner when needed!
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl AsRef<Secret<String>> for LoginAttemptId {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use secrecy::{ExposeSecret, Secret};

    #[tokio::test]
    async fn test_parse_login_attempt_id_ok() {
        let expected_value = "550e8400-e29b-41d4-a716-446655440000".to_string();
        let test_id = crate::domain::LoginAttemptId::parse(Secret::new(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
        ));
        assert!(test_id.is_ok());
        assert_eq!(
            expected_value,
            test_id.unwrap().0.expose_secret().to_string()
        )
    }

    #[tokio::test]
    async fn test_parse_login_attempt_id_err() {
        let expected_value = "Login Attempt ID doesn't match with UUID format".to_string();
        let test_id =
            crate::domain::LoginAttemptId::parse(Secret::new("invalid-uuid-format".to_string()));
        assert!(test_id.is_err());
        assert_eq!(expected_value, test_id.unwrap_err().to_string())
    }

    #[tokio::test]
    async fn test_default_login_attempt_id() {
        let test_id = crate::domain::LoginAttemptId::default();
        assert_eq!(36, test_id.0.expose_secret().len());
        assert_eq!('-', test_id.0.expose_secret().chars().nth(8).unwrap());
        assert_eq!('-', test_id.0.expose_secret().chars().nth(13).unwrap());
        assert_eq!('-', test_id.0.expose_secret().chars().nth(18).unwrap());
        assert_eq!('-', test_id.0.expose_secret().chars().nth(23).unwrap());
    }
}
