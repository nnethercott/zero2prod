use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::time::Duration;

use crate::{domain::SubscriberEmail, email_client::EmailClient};

#[derive(Clone, Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app: ApplicationSettings,
    pub email_client: EmailClientSettings,
    pub redis_uri: Secret<String>,
}

#[derive(Clone, Deserialize, Debug,)]
pub struct EmailClientSettings {
    pub sender_email: String,
    pub base_url: String,
    pub auth_token: Secret<String>,
    pub timeout_milliseconds: u64,
}
impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
    pub fn timeout(&self)->Duration{
        Duration::from_millis(self.timeout_milliseconds)
    }
    pub fn client(self)->Result<EmailClient, String>{
        let sender_email = self.sender()?;
        let timeout = self.timeout();

        Ok(EmailClient::new(
            self.base_url,
            sender_email,
            self.auth_token,
            timeout,
        ))
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub hmac_secret: Secret<String>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn connect_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
            .database(&self.database_name)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn to_string(&self) -> String {
        match self {
            Environment::Local => "local".to_string(),
            Environment::Production => "production".to_string(),
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            _ => Err(format!(
                "environment {} not found; use either `local` or `production`",
                s
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("failed to resolve current path");
    let configuration_dir = base_path.join("configuration");
    let environ: Environment = std::env::var("APP_ENVIRONMENT") // "local" or "production"
        .unwrap_or("local".to_string())
        .try_into()
        .expect("Failed to parse environment");
    let env_filename = format!("{}.yaml", environ.to_string());

    let settings = Config::builder()
        .add_source(File::from(configuration_dir.join("base.yaml")))
        .add_source(File::from(configuration_dir.join(env_filename)))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize()
}
