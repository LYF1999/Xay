use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub enum Encrypt {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "aes-128-gcm")]
    AES128GCM,
}
