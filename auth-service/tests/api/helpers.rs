use auth_service::{
    app_state::{AppState, BannedTokenStoreType, TwoFACodeStoreType},
    domain::Email,
    get_postgres_pool, get_redis_client,
    services::{
        PostgresUserStore, PostmarkEmailClient, RedisBannedTokenStore, RedisTwoFACodeStore,
    },
    utils::{test, DATABASE_URL, REDIS_HOST_NAME},
    Application,
};
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

use reqwest::{cookie::Jar, Client};
use secrecy::{ExposeSecret, Secret};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use tokio::sync::RwLock;
use uuid::Uuid;
use wiremock::MockServer;

// Global counter for Redis database selection (0-15 are available)
static REDIS_DB_COUNTER: AtomicU8 = AtomicU8::new(0);

pub struct TestApp {
    pub database_name: String,
    pub address: String,
    pub http_client: reqwest::Client,
    pub email_server: MockServer,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub redis_db: u8,
}

impl TestApp {
    pub async fn new() -> Self {
        // We are creating a new database for each test case, and we need to ensure each database has a unique name!
        let database_name = Uuid::new_v4().to_string();

        // Get a unique Redis database number (0-15) for this test
        let redis_db = REDIS_DB_COUNTER.fetch_add(1, Ordering::SeqCst) % 16;

        // In memory storage
        // let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
        // In DB storage
        let pg_pool = configure_postgresql(database_name.clone().to_string()).await;
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        // In memory storage
        // let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        // In REDIS storage
        let redis_conn = configure_redis(redis_db);
        let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn)));
        // In memory storage
        // let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        // In REDIS storage
        let redis_conn = configure_redis(redis_db);
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn)));
        // In memory email client
        // let email_client_type = Arc::new(RwLock::new(MockEmailClient::default()));
        // Mock email server
        let email_server = MockServer::start().await;
        let base_url = email_server.uri();
        let email_client_type = Arc::new(RwLock::new(configure_postmark_email_client(
            base_url.to_string(),
        )));

        let app_state = AppState::new(
            user_store,
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client_type.clone(),
        );

        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("Failed to build HTTP client.");

        Self {
            database_name: database_name,
            address: address,
            http_client: http_client,
            email_server: email_server,
            cookie_jar: cookie_jar,
            banned_token_store: banned_token_store,
            two_fa_code_store: two_fa_code_store,
            redis_db: redis_db,
        }
    }

    pub async fn clean_up(&self) {
        delete_database(&self.database_name).await;
        // Flush the specific Redis database to avoid test interference
        let mut redis_conn = configure_redis(self.redis_db);
        let _: Result<(), redis::RedisError> = redis::cmd("FLUSHDB").query(&mut redis_conn);
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .header("User-agent", "unit-tests")
            .header("Content-type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .header("User-agent", "unit-tests")
            .header("Content-type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .header("User-agent", "unit-tests")
            .header("Content-type", "application/json")
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .header("User-agent", "unit-tests")
            .header("Content-type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .header("User-agent", "unit-tests")
            .header("Content-type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub fn get_random_email() -> String {
    let uuid = Uuid::new_v4();

    format!("{}@example.com", uuid)
}

async fn configure_postgresql(db_name: String) -> PgPool {
    let postgresql_conn_url = DATABASE_URL.expose_secret().to_owned();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = Secret::new(format!("{}/{}", postgresql_conn_url, db_name));

    // Create a new connection pool and return it
    get_postgres_pool(postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let pool = PgPool::connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    let sql_statement = format!(r#"CREATE DATABASE "{}";"#, db_name);
    // Create a new database

    pool.execute(sql_statement.as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

fn configure_redis(db: u8) -> redis::Connection {
    let client = get_redis_client(REDIS_HOST_NAME.to_owned()).expect("Failed to get Redis client");

    let mut conn = client
        .get_connection()
        .expect("Failed to get Redis connection");

    // Select the specific database for this test
    redis::cmd("SELECT")
        .arg(db)
        .query::<()>(&mut conn)
        .expect("Failed to select Redis database");

    conn
}

fn configure_postmark_email_client(base_url: String) -> PostmarkEmailClient {
    let postmark_auth_token = Secret::new("auth_token".to_owned());

    let sender = Email::parse(test::email_client::SENDER.to_owned()).unwrap();

    let http_client = Client::builder()
        .timeout(test::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(base_url, sender, postmark_auth_token, http_client)
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: Secret<String> = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url.expose_secret())
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}
