#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), String>;
    async fn is_token_banned(&self, token: &str) -> Result<bool, String>;
}
