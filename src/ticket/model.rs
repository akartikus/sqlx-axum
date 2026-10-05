use time::OffsetDateTime;

#[derive(Debug)]
pub struct Ticket {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
}
