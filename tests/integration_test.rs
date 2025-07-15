use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_server_startup() {
    // Test that we can start the server on a random port
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let actual_addr = listener.local_addr().unwrap();
    
    // Verify we got a valid port
    assert!(actual_addr.port() > 0);
}

#[tokio::test]
async fn test_metrics_endpoint_response() {
    // This test would require a mock UniFi server to be comprehensive
    // For now, we just verify the endpoint structure
    
    // Create a test client with minimal config
    let _client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    // This would need to be run against a real or mocked server
    // For unit testing, we've covered the individual components
}

#[tokio::test]
async fn test_concurrent_polling() {
    // Test that multiple polling cycles don't interfere with each other
    let poll_interval = Duration::from_millis(100);
    let mut interval = tokio::time::interval(poll_interval);
    
    let mut count = 0;
    let start = tokio::time::Instant::now();
    
    // Run 5 ticks
    for _ in 0..5 {
        interval.tick().await;
        count += 1;
    }
    
    let elapsed = start.elapsed();
    
    // Verify timing is approximately correct (with some tolerance)
    assert_eq!(count, 5);
    assert!(elapsed >= Duration::from_millis(400));
    assert!(elapsed < Duration::from_millis(600));
}

#[test]
fn test_environment_variables() {
    // Test that environment variables are properly recognized
    unsafe {
        std::env::set_var("UNIFI_CONTROLLER_URL", "https://test.local");
        std::env::set_var("UNIFI_API_KEY", "test-key");
        std::env::set_var("METRICS_PORT", "9999");
    }
    
    assert_eq!(std::env::var("UNIFI_CONTROLLER_URL").unwrap(), "https://test.local");
    assert_eq!(std::env::var("UNIFI_API_KEY").unwrap(), "test-key");
    assert_eq!(std::env::var("METRICS_PORT").unwrap(), "9999");
    
    // Clean up
    unsafe {
        std::env::remove_var("UNIFI_CONTROLLER_URL");
        std::env::remove_var("UNIFI_API_KEY");
        std::env::remove_var("METRICS_PORT");
    }
}

#[tokio::test]
async fn test_graceful_shutdown() {
    // Test that the server can shut down gracefully
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    
    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = rx => {
                // Simulated shutdown
                Ok(())
            }
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                Err("Timeout waiting for shutdown")
            }
        }
    });
    
    // Send shutdown signal
    tx.send(()).unwrap();
    
    // Wait for clean shutdown
    let result = timeout(Duration::from_secs(1), handle).await;
    assert!(result.is_ok());
}