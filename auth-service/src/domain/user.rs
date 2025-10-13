use regex::Regex;

pub struct User {
    pub email: String,
    pub password: String,
    pub requires_2fa: bool
}

impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa
        }
    }

    pub fn validate_email(&self) -> bool {
        let re = Regex::new(r"(?i)^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}$").unwrap();
        re.is_match(&self.email)
    }

    pub fn validate_password(&self) -> bool {
        let re = Regex::new(r"^.{8,}$").unwrap();
        re.is_match(&self.password)
    }
}