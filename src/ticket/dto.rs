use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::model::Ticket;

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TicketResponse {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
}

impl From<Ticket> for TicketResponse {
    fn from(ticket: Ticket) -> Self {
        Self {
            id: ticket.id,
            title: ticket.title,
            description: ticket.description,
            created_at: ticket.created_at,
        }
    }
}
