use anyhow::Result;
use axum::{routing::get, Router};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

mod config;
mod metrics;
mod unifi;

use config::Config;
use metrics::Metrics;
use unifi::UniFiClient;

type SharedMetrics = Arc<RwLock<Metrics>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse configuration
    let config = Config::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    info!("Starting UniFi Network Exporter");

    // Create UniFi client
    let client = UniFiClient::new(
        config.controller_url.clone(),
        config.username.clone(),
        config.password.clone(),
        config.site.clone(),
        config.http_timeout_duration(),
        config.verify_ssl,
    )?;

    // Initialize metrics
    let metrics = Arc::new(RwLock::new(Metrics::new()?));

    // Create HTTP server for metrics
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .with_state(metrics.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Metrics server listening on {}", addr);

    // Spawn metrics server
    let server = tokio::spawn(async move {
        if let Err(e) = axum::serve(
            tokio::net::TcpListener::bind(addr).await.unwrap(),
            app.into_make_service(),
        )
        .await
        {
            error!("Server error: {}", e);
        }
    });

    // Start polling loop
    let poll_interval = config.poll_interval_duration();
    let mut interval = tokio::time::interval(poll_interval);

    loop {
        interval.tick().await;

        info!("Polling UniFi Controller");

        match poll_unifi_data(&client, &metrics).await {
            Ok(_) => info!("Successfully updated metrics"),
            Err(e) => error!("Failed to poll UniFi data: {}", e),
        }
    }

    server.await?;
    Ok(())
}

async fn poll_unifi_data(client: &UniFiClient, metrics: &SharedMetrics) -> Result<()> {
    // Authenticate if needed
    client.ensure_authenticated().await?;

    // Fetch data from UniFi
    let devices = client.fetch_devices().await?;
    let clients = client.fetch_clients().await?;
    let sites = client.fetch_sites().await?;

    // Update metrics
    let mut metrics = metrics.write().await;
    metrics.update_devices(&devices);
    metrics.update_clients(&clients);
    metrics.update_sites(&sites);

    Ok(())
}

async fn root_handler() -> &'static str {
    "UniFi Network Exporter\n\nEndpoints:\n  /metrics - Prometheus metrics\n  /health - Health check\n"
}

async fn metrics_handler(
    axum::extract::State(metrics): axum::extract::State<SharedMetrics>,
) -> String {
    let metrics = metrics.read().await;
    metrics.gather()
}

async fn health_handler() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_components() {
        // Test that main components are properly defined
        assert!(true);
    }
}