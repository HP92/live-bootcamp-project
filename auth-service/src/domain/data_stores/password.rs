use color_eyre::eyre::{eyre, Result};
use regex::Regex;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl Password {
    pub fn parse(password: Secret<String>) -> Result<Password> {
        if validate_password_requirements(password.expose_secret()) {
            Ok(Self(password))
        } else {
            Err(eyre!("Password doesn't fill the requirements"))
        }
    }

    pub fn value(&self) -> &str {
        self.0.expose_secret()
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        // We can use the expose_secret method to expose the secret in a
        // controlled manner when needed!
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

fn validate_password_requirements(password: &str) -> bool {
    let re = Regex::new(r"^.{8,}$").unwrap();
    re.is_match(password)
}

mod tests {
    #[tokio::test]
    async fn test_parse_password_ok() {
        let expected_value = "Asdf1234".to_string();
        let test_password =
            crate::domain::Password::parse(secrecy::Secret::new("Asdf1234".to_string()));

        assert!(test_password.is_ok());
        assert_eq!(expected_value, test_password.unwrap().value().to_string())
    }

    #[tokio::test]
    async fn test_parse_password_err() {
        let expected_value = "Password doesn't fill the requirements".to_string();

        let test_password =
            crate::domain::Password::parse(secrecy::Secret::new("Asdf123".to_string()));

        assert!(test_password.is_err());
        assert_eq!(expected_value, test_password.unwrap_err().to_string())
    }
}
