use std::{error::Error, sync::Arc};

use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    serve::Serve,
    Json, Router,
};

use reqwest::Method;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{services::hashmap_user_store::HashmapUserStore, utils::constants::DROPLET_IP};
use crate::{
    domain::AuthAPIError,
    routes::{login, logout, signup, verify_2fa, verify_token},
};

pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

pub type UserStoreType = Arc<RwLock<HashmapUserStore>>;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Incorrect credentials"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing token"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthAPIError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });

        (status, body).into_response()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}

pub struct Application {
    server: Serve<Router, Router>,
    // Address is pub so tests know it
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = vec![
            "http://localhost:8000".parse::<HeaderValue>()?,
            format!("http:{}//:8000", DROPLET_IP.to_string()).parse::<HeaderValue>()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            // Allow cookies to be included in requests
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .with_state(app_state)
            .layer(cors);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        let app = Application {
            server: server,
            address: address,
        };

        Ok(app)
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}
