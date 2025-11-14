use color_eyre::eyre::Result;

#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<()>;
    async fn contains_token(&mut self, token: &str) -> Result<bool>;
}
