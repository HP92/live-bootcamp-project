use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    app_state::AppState,
    domain::Email,
    get_postgres_pool, get_redis_client,
    services::{
        PostgresUserStore, PostmarkEmailClient, RedisBannedTokenStore, RedisTwoFACodeStore,
    },
    utils::{constants::prod, init_tracing, DATABASE_URL, POSTMARK_AUTH_TOKEN, REDIS_HOST_NAME},
    Application,
};

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");

    // In memory storage
    // let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    // In DB storage
    let pg_pool = configure_postgresql().await;
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    // In memory storage
    // let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    // In REDIS storage
    let redis_conn = configure_redis();
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn)));
    // In memory storage
    // let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    // In REDIS storage
    let redis_conn = configure_redis();
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn)));
    // In memory email client
    // let email_client_type = Arc::new(RwLock::new(MockEmailClient::default()));
    // In Postmark email client
    let email_client_type = Arc::new(RwLock::new(configure_postmark_email_client()));
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client_type,
    );

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(DATABASE_URL.to_owned())
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn configure_postmark_email_client() -> PostmarkEmailClient {
    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(
        prod::email_client::BASE_URL.to_owned(),
        Email::parse(prod::email_client::SENDER.to_owned()).unwrap(),
        POSTMARK_AUTH_TOKEN.to_owned(),
        http_client,
    )
}
