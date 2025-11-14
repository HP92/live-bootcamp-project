use std::hash::Hash;

use color_eyre::eyre::{eyre, Result};
use regex::Regex;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Email(Secret<String>);

impl Email {
    pub fn parse(address: Secret<String>) -> Result<Email> {
        if validate_email_requirements(address.expose_secret()) {
            Ok(Self(address))
        } else {
            Err(eyre!("Email doesn't fill the requirements"))
        }
    }

    pub fn value(&self) -> &str {
        self.0.expose_secret()
    }
}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        // We can use the expose_secret method to expose the secret in a
        // controlled manner when needed!
        self.0.expose_secret() == other.0.expose_secret()
    }
}

// Adding implementation of Eq for Email since PartialEq is implemented
impl Eq for Email {}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

fn validate_email_requirements(email: &str) -> bool {
    let re = Regex::new(r"(?i)^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$").unwrap();
    re.is_match(email)
}

mod tests {
    #[tokio::test]
    async fn test_parse_email_ok() {
        let expected_value = "test@example.com".to_string();
        let test_email =
            crate::domain::Email::parse(secrecy::Secret::new("test@example.com".to_string()));

        assert!(test_email.is_ok());
        assert_eq!(expected_value, test_email.unwrap().value().to_string())
    }

    #[tokio::test]
    async fn test_parse_email_err() {
        let expected_value = "Email doesn't fill the requirements".to_string();
        let test_email =
            crate::domain::Email::parse(secrecy::Secret::new("example.com".to_string()));

        assert!(test_email.is_err());
        assert_eq!(expected_value, test_email.unwrap_err().to_string())
    }
}
