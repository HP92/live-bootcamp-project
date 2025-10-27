use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        // Use the `parse_str` function from the `uuid` crate to ensure `id` is a valid UUID
        let result = Uuid::parse_str(&id);
        if result.is_ok() {
            Ok(Self(result.to_owned().unwrap().to_string()))
        } else {
            Err("Login Attempt ID doesn't match with UUID format".to_string())
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        // Use the `uuid` crate to generate a random version 4 UUID
        let uuid = Uuid::new_v4();
        Self(uuid.to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_parse_login_attempt_id_ok() {
        let expected_value = "550e8400-e29b-41d4-a716-446655440000".to_string();
        let test_id = crate::domain::LoginAttemptId::parse("550e8400-e29b-41d4-a716-446655440000".to_string());
        assert!(test_id.is_ok());
        assert_eq!(expected_value, test_id.unwrap().0)
    }   

    #[tokio::test]
    async fn test_parse_login_attempt_id_err() {
        let expected_value = "Login Attempt ID doesn't match with UUID format".to_string();
        let test_id = crate::domain::LoginAttemptId::parse("invalid-uuid-format".to_string());
        assert!(test_id.is_err());
        assert_eq!(expected_value, test_id.unwrap_err())
    }

    #[tokio::test]
    async fn test_default_login_attempt_id() {
        let test_id = crate::domain::LoginAttemptId::default();
        assert_eq!(36, test_id.0.len());
        assert_eq!('-', test_id.0.chars().nth(8).unwrap());
        assert_eq!('-', test_id.0.chars().nth(13).unwrap());
        assert_eq!('-', test_id.0.chars().nth(18).unwrap());
        assert_eq!('-', test_id.0.chars().nth(23).unwrap());
    }
}