use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

use super::handler;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tickets", post(handler::create).get(handler::list))
        .route("/tickets/{id}", get(handler::get_by_id))
}
