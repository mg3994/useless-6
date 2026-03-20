use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub secure: bool,
    pub from: String,
    pub auth: SmtpAuthConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SmtpAuthConfig {
    pub user: String,
    pub pass: String,
}