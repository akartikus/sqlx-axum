use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{state::AppState, ticket};

pub fn router(state: AppState) -> Router {
    Router::<AppState>::new()
        .nest("/api", ticket::routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
