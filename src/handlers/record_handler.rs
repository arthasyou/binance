use std::sync::Arc;

use axum::{extract::Query, Extension, Json};
use reqwest::StatusCode;

use super::get_api_key;
use crate::{
    binance::record_api::get_order_record_api,
    models::record_model::{GetOrderRequest, TradeRecord},
    secret_key::KeyManager,
};

pub async fn get_order(
    Extension(id): Extension<String>,
    Extension(api_keys): Extension<Arc<KeyManager>>,
    Query(params): Query<GetOrderRequest>,
) -> Result<Json<Vec<TradeRecord>>, (StatusCode, String)> {
    let key = get_api_key(api_keys, &id).await?;
    let data = get_order_record_api(
        &params.symbol,
        params.order_id,
        &key.api_key,
        &key.api_secret,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(data))
}
