use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

pub const JWT_COOKIE_NAME: &str = "jwt";

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const DROPLET_IP_ENV_VAR: &str = "DROPLET_IP";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}

lazy_static!{
    pub static ref JWT_SECRET: String = set_token();
    pub static ref DROPLET_IP: String = set_remote_ip();
}

fn set_token() -> String {
    dotenv().ok();
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT secret must be set in .env file");

    if secret.is_empty() {
        panic!("JWT secret cannot be empty");
    }

    secret
}

fn set_remote_ip() -> String {
    dotenv().ok();
    let remote_ip = std_env::var(env::DROPLET_IP_ENV_VAR).expect("DROPLET_IP must be set in .env file");

    if remote_ip.is_empty() {
        panic!("DROPLET_IP cannot be empty");
    }

    remote_ip
}