use axum::{routing::get, Router};

use crate::handlers::record_handler::get_order;

// use validator::Validate;

pub fn routes_record() -> Router {
    Router::new().route("/get_order", get(get_order))
}
