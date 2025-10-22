use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Password(String);

impl Password {
    pub fn parse(password: &str) -> Result<Password, String>{
        let re = Regex::new(r"^.{8,}$").unwrap();
        if re.is_match(password) {
            return Ok(Self(password.to_string()));
        } else {
            return Err("Password doesn't fill the requirements".to_string());
        }
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

mod tests {
    #[tokio::test]
    async fn test_parse_password_ok(){
        let expected_value = "Asdf1234".to_string();
        let test_password: Result<crate::domain::Password, String> = crate::domain::Password::parse("Asdf1234");

        assert!(test_password.is_ok());
        assert_eq!(expected_value, test_password.unwrap().0)
    }

    #[tokio::test]
    async fn test_parse_password_err(){
        let expected_value = "Password doesn't fill the requirements".to_string();
        let test_password = crate::domain::Password::parse("Asdf123");

        assert!(test_password.is_err());
        assert_eq!(expected_value, test_password.unwrap_err())
    }
}