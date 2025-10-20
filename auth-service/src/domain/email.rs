use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(address: &str) -> Result<Email, String>{
        let re = Regex::new(r"(?i)^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$").unwrap();
        if re.is_match(address) {
            return Ok(Self(address.to_string()));
        } else {
            return Err("Email doesn't fill the requirements".to_string());
        }
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

mod tests {
    #[tokio::test]
    async fn test_parse_email_ok(){
        let expected_value = "test@example.com".to_string();
        let test_email = crate::domain::Email::parse("test@example.com");

        assert!(test_email.is_ok());
        assert_eq!(expected_value, test_email.unwrap().0)
    }

    #[tokio::test]
    async fn test_parse_email_err(){
        let expected_value = "Email doesn't fill the requirements".to_string();
        let test_email = crate::domain::Email::parse("example.com");

        assert!(test_email.is_err());
        assert_eq!(expected_value, test_email.unwrap_err())
    }
}