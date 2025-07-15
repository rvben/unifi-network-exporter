use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// UniFi Controller URL (e.g., https://192.168.1.1:8443)
    #[arg(long, env = "UNIFI_CONTROLLER_URL")]
    pub controller_url: String,

    /// UniFi API key (use either API key or username/password)
    #[arg(long, env = "UNIFI_API_KEY")]
    pub api_key: Option<String>,

    /// UniFi Controller username (required if API key not provided)
    #[arg(long, env = "UNIFI_USERNAME")]
    pub username: Option<String>,

    /// UniFi Controller password (required if API key not provided)
    #[arg(long, env = "UNIFI_PASSWORD")]
    pub password: Option<String>,

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

    pub fn validate(&self) -> Result<(), String> {
        // Check that either API key or username/password is provided
        if self.api_key.is_none() && (self.username.is_none() || self.password.is_none()) {
            return Err(
                "Either UNIFI_API_KEY or both UNIFI_USERNAME and UNIFI_PASSWORD must be provided"
                    .to_string(),
            );
        }

        // Validate controller URL
        if self.controller_url.is_empty() {
            return Err("UNIFI_CONTROLLER_URL cannot be empty".to_string());
        }
        
        if !self.controller_url.starts_with("http://") && !self.controller_url.starts_with("https://") {
            return Err("UNIFI_CONTROLLER_URL must start with http:// or https://".to_string());
        }

        // Validate poll interval
        if self.poll_interval == 0 {
            return Err("POLL_INTERVAL must be greater than 0".to_string());
        }

        // Validate HTTP timeout
        if self.http_timeout == 0 {
            return Err("HTTP_TIMEOUT must be greater than 0".to_string());
        }

        // Validate port
        if self.port == 0 {
            return Err("METRICS_PORT cannot be 0".to_string());
        }

        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.to_lowercase().as_str()) {
            return Err(format!(
                "LOG_LEVEL must be one of: {}",
                valid_levels.join(", ")
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            controller_url: "https://192.168.1.1:8443".to_string(),
            api_key: None,
            username: Some("admin".to_string()),
            password: Some("password".to_string()),
            site: "default".to_string(),
            port: 9897,
            poll_interval: 30,
            log_level: "info".to_string(),
            http_timeout: 10,
            verify_ssl: true,
        }
    }

    #[test]
    fn test_poll_interval_duration() {
        let config = create_test_config();
        assert_eq!(config.poll_interval_duration(), Duration::from_secs(30));
    }

    #[test]
    fn test_http_timeout_duration() {
        let mut config = create_test_config();
        config.http_timeout = 15;
        assert_eq!(config.http_timeout_duration(), Duration::from_secs(15));
    }

    #[test]
    fn test_validate_with_api_key() {
        let mut config = create_test_config();
        config.api_key = Some("test-api-key".to_string());
        config.username = None;
        config.password = None;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_with_username_password() {
        let config = create_test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_missing_auth() {
        let mut config = create_test_config();
        config.api_key = None;
        config.username = None;
        config.password = None;
        assert!(config.validate().is_err());
        assert_eq!(
            config.validate().unwrap_err(),
            "Either UNIFI_API_KEY or both UNIFI_USERNAME and UNIFI_PASSWORD must be provided"
        );
    }

    #[test]
    fn test_validate_missing_password() {
        let mut config = create_test_config();
        config.api_key = None;
        config.password = None;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_default_values() {
        let config = Config {
            controller_url: "https://test.local".to_string(),
            api_key: Some("key".to_string()),
            username: None,
            password: None,
            site: "default".to_string(),
            port: 9897,
            poll_interval: 30,
            log_level: "info".to_string(),
            http_timeout: 10,
            verify_ssl: true,
        };
        assert_eq!(config.site, "default");
        assert_eq!(config.port, 9897);
        assert_eq!(config.poll_interval, 30);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.http_timeout, 10);
        assert_eq!(config.verify_ssl, true);
    }

    #[test]
    fn test_validate_empty_url() {
        let mut config = create_test_config();
        config.controller_url = "".to_string();
        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_invalid_url_scheme() {
        let mut config = create_test_config();
        config.controller_url = "ftp://test.local".to_string();
        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("must start with http:// or https://"));
    }

    #[test]
    fn test_validate_zero_poll_interval() {
        let mut config = create_test_config();
        config.poll_interval = 0;
        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("POLL_INTERVAL must be greater than 0"));
    }

    #[test]
    fn test_validate_zero_timeout() {
        let mut config = create_test_config();
        config.http_timeout = 0;
        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("HTTP_TIMEOUT must be greater than 0"));
    }

    #[test]
    fn test_validate_zero_port() {
        let mut config = create_test_config();
        config.port = 0;
        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("METRICS_PORT cannot be 0"));
    }

    #[test]
    fn test_validate_invalid_log_level() {
        let mut config = create_test_config();
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("LOG_LEVEL must be one of"));
    }

    #[test]
    fn test_validate_case_insensitive_log_level() {
        let mut config = create_test_config();
        config.log_level = "INFO".to_string();
        assert!(config.validate().is_ok());
        
        config.log_level = "Debug".to_string();
        assert!(config.validate().is_ok());
    }
}
