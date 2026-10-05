use sqlx::PgPool;

use crate::error::AppError;

use super::{dto::CreateTicketRequest, model::Ticket, repository};

pub async fn create(db: &PgPool, input: CreateTicketRequest) -> Result<Ticket, AppError> {
    let title = input.title.trim();

    if title.is_empty() {
        return Err(AppError::Validation("title cannot be empty".into()));
    }

    if title.chars().count() > 255 {
        return Err(AppError::Validation(
            "title cannot exceed 255 characters".into(),
        ));
    }

    Ok(repository::create(db, title, input.description.as_deref()).await?)
}

pub async fn get_by_id(db: &PgPool, id: i32) -> Result<Ticket, AppError> {
    repository::find_by_id(db, id)
        .await?
        .ok_or(AppError::NotFound)
}

pub async fn list(db: &PgPool) -> Result<Vec<Ticket>, AppError> {
    Ok(repository::find_all(db).await?)
}
