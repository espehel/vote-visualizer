use rust::{
    api::{AppState, router},
    storage::establish_connection,
    stortinget_client::StortingetClient,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = establish_connection(&database_url).await?;

    tracing::info!(
        size = pool.size(),
        idle_connections = pool.num_idle(),
        min_connections = pool.options().get_min_connections(),
        max_connections = pool.options().get_max_connections(),
        acquire_timeout = ?pool.options().get_acquire_timeout(),
        idle_timeout = ?pool.options().get_idle_timeout(),
        "database pool connected successfully"
    );

    let http = reqwest::Client::builder()
        .user_agent("vote-visualizer-importer/0.1")
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .build()?;

    let source = StortingetClient::new(http, "https://data.stortinget.no/eksport/".parse()?);
    let state = AppState {
        pool: pool.clone(),
        source,
    };
    let app = router().with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    let local_addr = listener.local_addr()?;
    tracing::info!(address = %local_addr, port = local_addr.port(), "server started successfully");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                tracing::error!(%error, "failed to listen for shutdown");
            }
        })
        .await?;

    pool.close().await;

    Ok(())
}
