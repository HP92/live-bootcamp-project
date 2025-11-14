use color_eyre::eyre::Result;
use secrecy::Secret;

#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: Secret<String>) -> Result<()>;
    async fn contains_token(&mut self, token: Secret<String>) -> Result<bool>;
}
