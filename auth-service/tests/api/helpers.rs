use std::sync::Arc;
use tokio::sync::RwLock;

use uuid::Uuid;

use auth_service::{services::HashmapUserStore, AppState, Application};

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(RwLock::new(HashmapUserStore::new()));
        let app_state = AppState::new(user_store);

        let app = Application::build(app_state,"127.0.0.1:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();

        let app = TestApp {
            address: address,
            http_client: http_client
        };

        app 
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
        Body: serde::Serialize 
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
    
    pub async fn post_login(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .header("User-agent", "unit-tests")
            .header("Content-type", "application/json")
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

    pub async fn post_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .header("User-agent", "unit-tests")
            .header("Content-type", "application/json")
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .header("User-agent", "unit-tests")
            .header("Content-type", "application/json")
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub fn get_random_email() -> String {
    let uuid = Uuid::new_v4();

    format!("{}@example.com", uuid)
}