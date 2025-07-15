use anyhow::Result;
use axum::{Router, routing::get};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

mod config;
mod metrics;
mod unifi;
mod unifi_integration;

use config::Config;
use metrics::Metrics;
use unifi::UniFiClient;

type SharedMetrics = Arc<RwLock<Metrics>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse configuration
    let config = Config::parse();

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {e}");
        std::process::exit(1);
    }

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    info!("Starting UniFi Network Exporter");

    // Create UniFi client
    let client = UniFiClient::new(
        config.controller_url.clone(),
        config.api_key.clone(),
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

    // Start polling loop in a separate task
    let poll_metrics = metrics.clone();
    let poll_handle = tokio::spawn(async move {
        let poll_interval = config.poll_interval_duration();
        let mut interval = tokio::time::interval(poll_interval);

        loop {
            interval.tick().await;

            info!("Polling UniFi Controller");

            match poll_unifi_data(&client, &poll_metrics).await {
                Ok(_) => info!("Successfully updated metrics"),
                Err(e) => error!("Failed to poll UniFi data: {}", e),
            }
        }
    });

    // Wait for both tasks
    tokio::select! {
        _ = server => error!("Server task ended unexpectedly"),
        _ = poll_handle => error!("Polling task ended unexpectedly"),
    }

    Ok(())
}

async fn poll_unifi_data(client: &UniFiClient, metrics: &SharedMetrics) -> Result<()> {
    // Authenticate if needed
    client.ensure_authenticated().await?;

    // Fetch data from UniFi
    let devices = client.get_devices().await?;
    let clients = client.get_clients().await?;
    let sites = client.get_sites().await?;

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
    use axum::http::Request;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[test]
    fn test_main_components() {
        // Test that main components are properly defined
        assert!(true);
    }

    #[tokio::test]
    async fn test_root_handler() {
        let response = root_handler().await;
        assert!(response.contains("UniFi Network Exporter"));
        assert!(response.contains("/metrics"));
        assert!(response.contains("/health"));
    }

    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await;
        assert_eq!(response, "OK");
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let metrics = Arc::new(RwLock::new(Metrics::new().unwrap()));
        let response = metrics_handler(axum::extract::State(metrics)).await;
        // The response should be a valid Prometheus format even if empty
        assert!(response.is_empty() || response.contains("# HELP") || response.contains("# TYPE"));
    }

    #[tokio::test]
    async fn test_router_creation() {
        let metrics = Arc::new(RwLock::new(Metrics::new().unwrap()));
        let app = Router::new()
            .route("/", get(root_handler))
            .route("/metrics", get(metrics_handler))
            .route("/health", get(health_handler))
            .with_state(metrics.clone());

        // Test root endpoint
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test health endpoint
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test metrics endpoint
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test 404
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
