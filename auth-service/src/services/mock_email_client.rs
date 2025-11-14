use color_eyre::eyre::Result;
use thiserror::Error;

use crate::domain::{Email, EmailClient};

#[derive(Debug, Default, Clone, Error)]
pub struct MockEmailClient;

// Implement Display for MockEmailClient
impl std::fmt::Display for MockEmailClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockEmailClient")
    }
}

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()> {
        // Our mock email client will simply log the recipient, subject, and content to standard output
        println!(
            "Sending email to {} with subject: {} and content: {}",
            recipient.as_ref(),
            subject,
            content
        );

        Ok(())
    }
}
