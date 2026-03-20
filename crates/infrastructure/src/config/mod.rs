use serde::Deserialize;
pub mod db_config;
pub mod log_config;
pub mod smtp_config;
pub mod s3_config;

use std::sync::OnceLock;
use figment::Figment;
use figment::providers::{Env, Format, Toml};
//

pub use log_config::LogConfig;

//
pub use db_config::DbConfig;

pub use smtp_config::SmtpConfig;
use s3_config::S3Config;

pub static CONFIG: OnceLock<ServerConfig> = OnceLock::new();
pub fn init()  {
    let raw_config = Figment::new()
        .merge(Toml::file(
            std::env::var("APP_CONFIG").as_deref().unwrap_or("config.toml"),
        ))
        .merge(Env::prefixed("APP_").global());
//
    let mut config = match raw_config.extract::<ServerConfig>() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("It looks like your config is invalid. The following error occurred: {e}");
            std::process::exit(1);
        }
    };
    if config.db.url.is_empty() {
        config.db.url = std::env::var("DATABASE_URL").unwrap_or_default();
    }
    if config.db.url.is_empty() {
        eprintln!("DATABASE_URL is not set");
        std::process::exit(1);
    }
   CONFIG
        .set(config)
        .expect("config should be set");
//
}
//
pub fn get() -> &'static ServerConfig {
   CONFIG.get().expect("config should be set")
}
//
//
//
#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    // not being used
    #[serde(default = "default_locale")]
    pub default_locale: String,

    pub db: DbConfig,
    pub log: LogConfig,

    pub tls: Option<TlsConfig>,



    pub smtp: SmtpConfig,
    pub s3: S3Config, // 👈 add this

}


#[derive(Deserialize, Clone, Debug)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
}



#[allow(dead_code)]
pub fn default_false() -> bool {
    false
}
#[allow(dead_code)]
pub fn default_true() -> bool {
    true
}
#[allow(dead_code)]
fn default_listen_addr() -> String {
    "127.0.0.1:8008".into()
}

#[allow(dead_code)]
fn default_locale() -> String {
    "en".into()
}

//

#[allow(dead_code)]
fn default_helper_threads() -> usize {
    10
}
#[allow(dead_code)]
fn default_db_pool_size() -> u32 {
    10
}
#[allow(dead_code)]
fn default_tcp_timeout() -> u64 {
    10000
}
#[allow(dead_code)]
fn default_connection_timeout() -> u64 {
    30000
}
#[allow(dead_code)]
fn default_statement_timeout() -> u64 {
    30000
}

//
#[allow(dead_code)]
fn default_filter_level() -> String {
    "info".into()
}
#[allow(dead_code)]
fn default_directory() -> String {
    "./logs".into()
}
#[allow(dead_code)]
fn default_file_name() -> String {
    "app.log".into()
}


#[allow(dead_code)]
fn default_rolling() -> String {
    "daily".into()
}
#[allow(dead_code)]
fn default_format() -> String {
    FORMAT_FULL.into()
}
//


pub const FORMAT_PRETTY: &str = "pretty";
pub const FORMAT_COMPACT: &str = "compact";
pub const FORMAT_JSON: &str = "json";
pub const FORMAT_FULL: &str = "full";