use axum::{
    routing::{delete, get, post},
    Router,
};

// use validator::Validate;

use crate::handlers::trade_hander::{
    close_trade, create_trade, delete_trade_by_id, get_adjustments, get_all_history_trades,
    get_price, get_trade, get_user_hold, update_adjustments,
};

pub fn routes_trade() -> Router {
    Router::new()
        .route("/create_trade", post(create_trade))
        .route("/close_trade", post(close_trade))
        .route("/get_trade", get(get_trade))
        .route("/get_price", get(get_price))
        .route("/get_all_history_trades", get(get_all_history_trades))
        .route("/delete_trade", delete(delete_trade_by_id))
        .route("/get_adjustments", get(get_adjustments))
        .route("/update_adjustments", post(update_adjustments))
        .route("/get_hold", get(get_user_hold))
}
