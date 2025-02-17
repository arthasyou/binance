use std::sync::Arc;

use reqwest::StatusCode;

use crate::secret_key::{KeyManager, SecretKey};

pub mod auth_handler;
pub mod record_handler;
pub mod trade_hander;

pub async fn get_api_key(
    api_keys: Arc<KeyManager>,
    id: &str,
) -> Result<SecretKey, (StatusCode, String)> {
    match api_keys.get_key(id) {
        Some(key) => Ok(key),
        None => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get API key".to_owned(),
        )),
    }
}
