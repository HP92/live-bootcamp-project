use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
            Ok(Self(code))
        } else {
            Err("2FA Code must be a 6-digit number".to_string())
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        // Use the `rand` crate to generate a random 2FA code.
        // The code should be 6 digits (ex: 834629)
        let mut rng = thread_rng();

        // Generate a 6-character numeric string
        let code: String = (0..6).map(|_| rng.gen_range(0..10).to_string()).collect();

        Self(code)
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_parse_2fa_code_ok() {
        let expected_value = "123456".to_string();
        let test_code = crate::domain::TwoFACode::parse("123456".to_string());
        assert!(test_code.is_ok());
        assert_eq!(expected_value, test_code.unwrap().0)
    }

    #[tokio::test]
    async fn test_parse_2fa_code_err() {
        let expected_value = "2FA Code must be a 6-digit number".to_string();
        let test_code = crate::domain::TwoFACode::parse("12345a".to_string());
        assert!(test_code.is_err());
        assert_eq!(expected_value, test_code.unwrap_err())
    }

    #[tokio::test]
    async fn test_default_2fa_code() {
        let test_code = crate::domain::TwoFACode::default();
        assert_eq!(6, test_code.0.len());
        assert!(test_code.0.chars().all(|c| c.is_digit(10)));
    }
}
