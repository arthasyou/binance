mod auth_route;
pub mod error;
mod trade_route;

use std::{collections::HashMap, sync::Arc};

use crate::{
    mw::{auth_mw, cors::create_cors},
    secret_key::KeyManager,
    trade::{AdjustmentConfig, Trade},
    utils::TradeIdGenerator,
};

use auth_route::routes_auth;
use axum::{middleware, Extension, Router};

use sea_orm::DatabaseConnection;
use service_utils_rs::services::jwt::Jwt;
use tokio::sync::Mutex;
use trade_route::routes_trade;

pub fn create_routes(
    trads: Arc<HashMap<String, Mutex<Vec<Trade>>>>,
    prices: Arc<HashMap<String, Mutex<(String, String)>>>,
    id_generator: Arc<TradeIdGenerator>,
    database: DatabaseConnection,
    precisions: Arc<HashMap<String, u8>>,
    adjustment: Arc<HashMap<u8, Mutex<AdjustmentConfig>>>,
    jwt: Jwt,
    api_keys: Arc<KeyManager>,
) -> Router {
    let cors = create_cors();

    Router::new()
        // .merge(routes_manage())
        .nest("/trade", routes_trade())
        .route_layer(middleware::from_fn(auth_mw::auth))
        .nest("/auth", routes_auth())
        .layer(Extension(trads))
        .layer(Extension(prices))
        .layer(Extension(id_generator))
        .layer(Extension(precisions))
        .layer(Extension(adjustment))
        .layer(Extension(database))
        .layer(Extension(jwt))
        .layer(Extension(api_keys))
        .layer(cors)
}
