mod error;
mod trade_route;

use std::{collections::HashMap, sync::Arc};

use crate::{
    mw::cors::create_cors,
    trade::{AdjustmentConfig, Trade},
    utils::TradeIdGenerator,
};

use axum::{Extension, Router};

use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use trade_route::routes_trade;

pub fn create_routes(
    trads: Arc<HashMap<String, Mutex<Vec<Trade>>>>,
    prices: Arc<HashMap<String, Mutex<(String, String)>>>,
    id_generator: Arc<TradeIdGenerator>,
    database: DatabaseConnection,
    precisions: Arc<HashMap<String, u8>>,
    adjustment: Arc<HashMap<u8, Mutex<AdjustmentConfig>>>,
) -> Router {
    let cors = create_cors();

    Router::new()
        // .merge(routes_manage())
        .nest("/trade", routes_trade())
        .layer(Extension(trads))
        .layer(Extension(prices))
        .layer(Extension(id_generator))
        .layer(Extension(precisions))
        .layer(Extension(adjustment))
        .layer(Extension(database))
        .layer(cors)
}
