use sqlx::postgres::PgExecutor;

use super::model::Ticket;

pub async fn create<'e, E>(
    executor: E,
    title: &str,
    description: Option<&str>,
) -> Result<Ticket, sqlx::Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Ticket,
        r#"
        INSERT INTO tickets (
            title,
            description
        )
        VALUES ($1, $2)
        RETURNING
            id,
            title,
            description,
            created_at
        "#,
        title,
        description,
    )
    .fetch_one(executor)
    .await
}

pub async fn find_by_id<'e, E>(executor: E, id: i32) -> Result<Option<Ticket>, sqlx::Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Ticket,
        r#"
        SELECT
            id,
            title,
            description,
            created_at
        FROM tickets
        WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(executor)
    .await
}

pub async fn find_all<'e, E>(executor: E) -> Result<Vec<Ticket>, sqlx::Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Ticket,
        r#"
        SELECT
            id,
            title,
            description,
            created_at
        FROM tickets
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(executor)
    .await
}
