#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}
