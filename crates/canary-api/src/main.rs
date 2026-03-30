use canary_api::{create_app, AppConfig};
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = AppConfig {
        database_url: env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set"),
        port: env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()?,
    };

    tracing::info!("Initializing database...");
    let app = create_app(config.clone()).await?;

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
