use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// UniFi Controller URL (e.g., https://192.168.1.1:8443)
    #[arg(long, env = "UNIFI_CONTROLLER_URL")]
    pub controller_url: String,

    /// UniFi username
    #[arg(long, env = "UNIFI_USERNAME")]
    pub username: String,

    /// UniFi password
    #[arg(long, env = "UNIFI_PASSWORD")]
    pub password: String,

    /// UniFi site name (default: 'default')
    #[arg(long, env = "UNIFI_SITE", default_value = "default")]
    pub site: String,

    /// Port to expose metrics on
    #[arg(short, long, env = "METRICS_PORT", default_value = "9897")]
    pub port: u16,

    /// Poll interval in seconds
    #[arg(long, env = "POLL_INTERVAL", default_value = "30")]
    pub poll_interval: u64,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    /// HTTP timeout in seconds
    #[arg(long, env = "HTTP_TIMEOUT", default_value = "10")]
    pub http_timeout: u64,

    /// Verify SSL certificates
    #[arg(long, env = "VERIFY_SSL", default_value = "true")]
    pub verify_ssl: bool,
}

impl Config {
    pub fn poll_interval_duration(&self) -> Duration {
        Duration::from_secs(self.poll_interval)
    }

    pub fn http_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.http_timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_interval_duration() {
        let config = Config {
            controller_url: "https://192.168.1.1:8443".to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
            site: "default".to_string(),
            port: 9897,
            poll_interval: 30,
            log_level: "info".to_string(),
            http_timeout: 10,
            verify_ssl: true,
        };
        assert_eq!(config.poll_interval_duration(), Duration::from_secs(30));
    }

    #[test]
    fn test_http_timeout_duration() {
        let config = Config {
            controller_url: "https://192.168.1.1:8443".to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
            site: "default".to_string(),
            port: 9897,
            poll_interval: 30,
            log_level: "info".to_string(),
            http_timeout: 15,
            verify_ssl: true,
        };
        assert_eq!(config.http_timeout_duration(), Duration::from_secs(15));
    }
}