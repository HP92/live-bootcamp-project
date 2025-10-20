use std::{
    error::Error,
    sync::Arc
};

use axum::{ 
    http::StatusCode, 
    response::{IntoResponse, Response}, 
    routing::post, 
    serve::Serve, 
    Json, 
    Router
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::domain::AuthAPIError;
use crate::services::hashmap_user_store::HashmapUserStore;

pub mod domain;
pub mod routes;
pub mod services;

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
            AuthAPIError::UnexpectedError => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Incorrect credentials"),
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string()
        });
        
        (status, body).into_response()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self {
            user_store
        }
    }
}

pub struct Application {
    server: Serve<Router, Router>,
    // Address is pub so tests know it
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
        .nest_service("/", ServeDir::new("assets"))
        .route("/signup", post(routes::signup))
        .route("/login", post(routes::login))
        .route("/logout", post(routes::logout))
        .route("/verify-2fa", post(routes::verify_2fa))
        .route("/verify-token", post(routes::verify_token))
        .with_state(app_state.into());

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        let app = Application {
            server: server,
            address: address
        };

        Ok(app)
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}










