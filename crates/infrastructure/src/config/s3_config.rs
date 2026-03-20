use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize, Clone, Debug)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
}
