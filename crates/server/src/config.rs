use std::{
    env,
    net::{IpAddr, SocketAddr},
};

use anyhow::{Context, Result};
use axum::http::{
    HeaderValue, Method,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let server_host = env_string("SERVER_HOST", "0.0.0.0");

        server_host.parse::<IpAddr>().with_context(|| {
            format!("SERVER_HOST must be a valid IP address, got '{server_host}'")
        })?;

        let server_port = env_port("SERVER_PORT", 8080)?;

        let cors_allowed_origins = env_list(
            "CORS_ALLOWED_ORIGINS",
            &[
                "http://localhost:3000".to_string(),
                "http://localhost:5173".to_string(),
            ],
        );

        if cors_allowed_origins.is_empty() {
            anyhow::bail!("CORS_ALLOWED_ORIGINS must contain at least one origin");
        }

        for origin in &cors_allowed_origins {
            origin.parse::<HeaderValue>().with_context(|| {
                format!("CORS_ALLOWED_ORIGINS contains an invalid origin: '{origin}'")
            })?;
        }

        Ok(Self {
            server_host,
            server_port,
            cors_allowed_origins,
        })
    }

    pub fn bind_address(&self) -> SocketAddr {
        let ip: IpAddr = self
            .server_host
            .parse()
            .expect("SERVER_HOST must contain a valid IP address");

        SocketAddr::from((ip, self.server_port))
    }
}

fn env_string(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_port(name: &str, default: u16) -> Result<u16> {
    let value = match env::var(name) {
        Ok(value) => value,
        Err(_) => return Ok(default),
    };

    let port = value
        .parse::<u16>()
        .with_context(|| format!("{name} must be a valid port number, got '{value}'"))?;

    if !(1024..=65535).contains(&port) {
        anyhow::bail!("{name} must be between 1024 and 65535, got {port}");
    }

    Ok(port)
}

fn env_list(name: &str, default: &[String]) -> Vec<String> {
    match env::var(name) {
        Ok(value) => value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        Err(_) => default.to_vec(),
    }
}

pub fn build_cors(origins: &[String]) -> CorsLayer {
    let origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
}
