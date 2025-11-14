use color_eyre::eyre::{eyre, Result};
use rand::{thread_rng, Rng};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TwoFACode(Secret<String>);

impl TwoFACode {
    pub fn parse(code: Secret<String>) -> Result<Self> {
        // Check if the code is exactly 6 characters
        if code.expose_secret().len() != 6 {
            return Err(eyre!("2FA Code must be a 6-digit number"));
        }

        // Check if all characters are digits
        if !code.expose_secret().chars().all(|c| c.is_ascii_digit()) {
            return Err(eyre!("2FA Code must be a number"));
        }

        Ok(Self(code))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        // Use the `rand` crate to generate a random 2FA code.
        // The code should be 6 digits (ex: 834629)
        let mut rng = thread_rng();

        // Generate a 6-character numeric string
        let code = Secret::new((0..6).map(|_| rng.gen_range(0..10).to_string()).collect());

        Self(code)
    }
}

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        // We can use the expose_secret method to expose the secret in a
        // controlled manner when needed!
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl AsRef<Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    #[tokio::test]
    async fn test_parse_2fa_code_ok() {
        let expected_value = "123456".to_string();
        let test_code = crate::domain::TwoFACode::parse(secrecy::Secret::new("123456".to_string()));
        assert!(test_code.is_ok());
        assert_eq!(
            expected_value,
            test_code.unwrap().0.expose_secret().to_string()
        )
    }

    #[tokio::test]
    async fn test_parse_2fa_code_err() {
        let expected_value = "2FA Code must be a number".to_string();
        let test_code = crate::domain::TwoFACode::parse(secrecy::Secret::new("12345a".to_string()));
        assert!(test_code.is_err());
        assert_eq!(expected_value, test_code.unwrap_err().to_string())
    }

    #[tokio::test]
    async fn test_default_2fa_code() {
        let test_code = crate::domain::TwoFACode::default();
        assert_eq!(6, test_code.0.expose_secret().len());
        assert!(test_code.0.expose_secret().chars().all(|c| c.is_digit(10)));
    }
}
