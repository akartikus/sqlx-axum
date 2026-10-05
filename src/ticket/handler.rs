use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{error::AppError, state::AppState};

use super::{
    dto::{CreateTicketRequest, TicketResponse},
    service,
};

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateTicketRequest>,
) -> Result<(StatusCode, Json<TicketResponse>), AppError> {
    let ticket = service::create(&state.db, input).await?;

    Ok((StatusCode::CREATED, Json(ticket.into())))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<TicketResponse>, AppError> {
    let ticket = service::get_by_id(&state.db, id).await?;

    Ok(Json(ticket.into()))
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TicketResponse>>, AppError> {
    let tickets = service::list(&state.db).await?;

    let response = tickets.into_iter().map(TicketResponse::from).collect();

    Ok(Json(response))
}
