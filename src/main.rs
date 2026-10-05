use ticket_api::{app, config::Config, db, state::AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;

    let db = db::connect(&config.database_url).await?;
    db::migrate(&db).await?;

    let state = AppState { db };
    let app = app::router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("server listening on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
