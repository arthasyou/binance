use axum::{routing::post, Router};

use crate::handlers::auth_handler::{login, signup};

pub fn routes_auth() -> Router {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
}
