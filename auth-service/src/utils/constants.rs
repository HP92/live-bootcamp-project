use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DROPLET_IP_ENV_VAR: &str = "DROPLET_IP";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_URL";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref DROPLET_IP: String = set_remote_ip();
    pub static ref DATABASE_URL: String = set_database_url();
    pub static ref REDIS_HOST_NAME: String = set_redis_host();
}

fn set_token() -> String {
    dotenv().ok();
    let secret =
        std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT secret must be set in .env file");

    if secret.is_empty() {
        panic!("JWT secret cannot be empty");
    }

    secret
}

fn set_remote_ip() -> String {
    dotenv().ok();
    let remote_ip =
        std_env::var(env::DROPLET_IP_ENV_VAR).expect("DROPLET_IP must be set in .env file");

    if remote_ip.is_empty() {
        panic!("DROPLET_IP cannot be empty");
    }

    remote_ip
}

fn set_database_url() -> String {
    dotenv().ok();
    let database_url =
        std_env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set in .env file");

    if database_url.is_empty() {
        panic!("DATABASE_URL cannot be empty");
    }

    database_url
}

fn set_redis_host() -> String {
    dotenv().ok();
    std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned())
}
