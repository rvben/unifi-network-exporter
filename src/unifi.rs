use anyhow::{Result, anyhow};
use reqwest::header::{ACCEPT, COOKIE, HeaderMap, HeaderValue};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::debug;

use crate::unifi_integration::{IntegrationResponse, IntegrationSite};

// Helper function to deserialize optional string to f64
fn deserialize_optional_string_to_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    // This handles fields that might be missing entirely
    let opt: Option<Option<String>> = Option::deserialize(deserializer)?;
    match opt {
        Some(Some(s)) => s.parse::<f64>()
            .map(Some)
            .map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

#[derive(Error, Debug)]
pub enum UniFiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
    remember: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginResponse {
    meta: Meta,
    data: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Meta {
    rc: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Device {
    pub _id: String,
    pub name: Option<String>,
    pub mac: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub model: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub adopted: bool,
    #[serde(default)]
    pub state: i32,
    pub uptime: Option<i64>,
    pub sys_stats: Option<SysStats>,
    pub stat: Option<DeviceStats>,
    
    // Catch-all for additional fields from the API
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SysStats {
    #[serde(default, deserialize_with = "deserialize_optional_string_to_f64")]
    pub loadavg_1: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_string_to_f64")]
    pub loadavg_5: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_string_to_f64")]
    pub loadavg_15: Option<f64>,
    pub mem_total: Option<i64>,
    pub mem_used: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct DeviceStats {
    pub bytes: Option<i64>,
    pub tx_bytes: Option<i64>,
    pub rx_bytes: Option<i64>,
    pub tx_packets: Option<i64>,
    pub rx_packets: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Client {
    pub _id: String,
    pub mac: String,
    pub ip: Option<String>,
    pub hostname: Option<String>,
    pub name: Option<String>,
    pub network: Option<String>,
    pub vlan: Option<i32>,
    pub ap_mac: Option<String>,
    pub signal: Option<i32>,
    pub tx_bytes: Option<i64>,
    pub rx_bytes: Option<i64>,
    pub uptime: Option<i64>,
    #[serde(default)]
    pub is_wired: bool,
    #[serde(default)]
    pub is_guest: bool,
    
    // Catch-all for additional fields from the API
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Site {
    pub _id: String,
    pub name: String,
    pub desc: String,
    pub attr_hidden_id: Option<String>,
    pub attr_no_delete: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiResponse<T> {
    meta: Meta,
    data: Vec<T>,
}

#[derive(Clone)]
enum AuthMethod {
    ApiKey(String),
    UserPass { username: String, password: String },
}

pub struct UniFiClient {
    client: reqwest::Client,
    base_url: String,
    auth_method: AuthMethod,
    site: String,
    auth_cookies: Arc<RwLock<Option<String>>>,
}

impl UniFiClient {
    pub fn new(
        base_url: String,
        api_key: Option<String>,
        username: Option<String>,
        password: Option<String>,
        site: String,
        timeout: Duration,
        verify_ssl: bool,
    ) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .danger_accept_invalid_certs(!verify_ssl)
            .cookie_store(true)
            .build()?;

        // Determine auth method
        let auth_method = if let Some(key) = api_key {
            AuthMethod::ApiKey(key)
        } else if let (Some(user), Some(pass)) = (username, password) {
            AuthMethod::UserPass {
                username: user,
                password: pass,
            }
        } else {
            return Err(anyhow!(
                "Either API key or username/password must be provided"
            ));
        };

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_method,
            site,
            auth_cookies: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn ensure_authenticated(&self) -> Result<()> {
        match &self.auth_method {
            AuthMethod::ApiKey(_) => Ok(()), // API key doesn't need login
            AuthMethod::UserPass { .. } => {
                let cookies = self.auth_cookies.read().await;
                if cookies.is_some() {
                    return Ok(());
                }
                drop(cookies);
                self.login().await
            }
        }
    }

    async fn login(&self) -> Result<()> {
        match &self.auth_method {
            AuthMethod::ApiKey(_) => Ok(()), // No login needed for API key
            AuthMethod::UserPass { username, password } => {
                let login_url = format!("{}/api/login", self.base_url);
                let login_data = LoginRequest {
                    username: username.clone(),
                    password: password.clone(),
                    remember: false,
                };

                let response = self
                    .client
                    .post(&login_url)
                    .json(&login_data)
                    .send()
                    .await?;

                if !response.status().is_success() {
                    return Err(anyhow!("Login failed with status: {}", response.status()));
                }

                // Extract cookies from response
                let cookies: Vec<String> = response
                    .headers()
                    .get_all("set-cookie")
                    .iter()
                    .filter_map(|value| value.to_str().ok())
                    .map(|s| s.to_string())
                    .collect();

                if cookies.is_empty() {
                    return Err(anyhow!("No cookies received from login response"));
                }

                let cookie_string = cookies.join("; ");
                *self.auth_cookies.write().await = Some(cookie_string);

                Ok(())
            }
        }
    }

    async fn get_legacy<T>(&self, path: &str) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = match &self.auth_method {
            AuthMethod::ApiKey(_) => {
                // API key uses different URL pattern
                format!(
                    "{}/proxy/network/integration/v1/{}",
                    self.base_url,
                    path.trim_start_matches('/')
                )
            }
            AuthMethod::UserPass { .. } => {
                // Cookie auth uses traditional API path
                format!(
                    "{}/api/s/{}/{}",
                    self.base_url,
                    self.site,
                    path.trim_start_matches('/')
                )
            }
        };

        debug!("Making request to: {}", url);

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        match &self.auth_method {
            AuthMethod::ApiKey(key) => {
                headers.insert("X-API-KEY", HeaderValue::from_str(key).unwrap());
            }
            AuthMethod::UserPass { .. } => {
                if let Some(cookies) = &*self.auth_cookies.read().await {
                    headers.insert(COOKIE, HeaderValue::from_str(cookies).unwrap());
                }
            }
        }

        let response = self.client.get(&url).headers(headers).send().await?;

        if response.status() == 401 && matches!(&self.auth_method, AuthMethod::UserPass { .. }) {
            // Try to re-authenticate
            drop(self.auth_cookies.write().await.take());
            self.login()
                .await
                .map_err(|_| UniFiError::AuthenticationFailed)?;

            // Retry request
            let mut headers = HeaderMap::new();
            headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
            if let Some(cookies) = &*self.auth_cookies.read().await {
                headers.insert(COOKIE, HeaderValue::from_str(cookies).unwrap());
            }

            let response = self.client.get(&url).headers(headers).send().await?;

            if !response.status().is_success() {
                return Err(UniFiError::ParseError(format!(
                    "API request failed with status: {}",
                    response.status()
                ))
                .into());
            }

            let api_response: ApiResponse<T> = response.json().await?;
            Ok(api_response.data)
        } else if response.status().is_success() {
            let api_response: ApiResponse<T> = response.json().await?;
            Ok(api_response.data)
        } else {
            Err(UniFiError::ParseError(format!(
                "API request failed with status: {}",
                response.status()
            ))
            .into())
        }
    }

    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        match &self.auth_method {
            AuthMethod::ApiKey(key) => {
                // Use the regular API with API key authentication for full metrics
                let url = format!(
                    "{}/proxy/network/api/s/{}/stat/device",
                    self.base_url, self.site
                );

                debug!("Making request to: {}", url);

                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                headers.insert("X-API-KEY", HeaderValue::from_str(key)?);

                let response = self.client.get(&url).headers(headers).send().await?;

                if !response.status().is_success() {
                    return Err(anyhow!("API request failed: {}", response.status()));
                }

                #[derive(Debug, Deserialize)]
                struct ApiResponse {
                    #[allow(dead_code)]
                    meta: Meta,
                    data: Vec<Device>,
                }

                let text = response.text().await?;
                match serde_json::from_str::<ApiResponse>(&text) {
                    Ok(api_response) => Ok(api_response.data),
                    Err(e) => {
                        eprintln!("Failed to parse device JSON: {}", e);
                        eprintln!("Response text (first 500 chars): {}", &text.chars().take(500).collect::<String>());
                        Err(anyhow!("Failed to parse device response: {}", e))
                    }
                }
            }
            AuthMethod::UserPass { .. } => self.get_legacy("stat/device").await,
        }
    }

    pub async fn get_clients(&self) -> Result<Vec<Client>> {
        match &self.auth_method {
            AuthMethod::ApiKey(key) => {
                // Use the regular API with API key authentication for full metrics
                let url = format!(
                    "{}/proxy/network/api/s/{}/stat/sta",
                    self.base_url, self.site
                );

                debug!("Making request to: {}", url);

                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                headers.insert("X-API-KEY", HeaderValue::from_str(key)?);

                let response = self.client.get(&url).headers(headers).send().await?;

                if !response.status().is_success() {
                    return Err(anyhow!("API request failed: {}", response.status()));
                }

                #[derive(Debug, Deserialize)]
                struct ApiResponse {
                    #[allow(dead_code)]
                    meta: Meta,
                    data: Vec<Client>,
                }

                let text = response.text().await?;
                match serde_json::from_str::<ApiResponse>(&text) {
                    Ok(api_response) => Ok(api_response.data),
                    Err(e) => {
                        eprintln!("Failed to parse client JSON: {}", e);
                        eprintln!("Response text (first 500 chars): {}", &text.chars().take(500).collect::<String>());
                        Err(anyhow!("Failed to parse client response: {}", e))
                    }
                }
            }
            AuthMethod::UserPass { .. } => self.get_legacy("stat/sta").await,
        }
    }

    pub async fn get_sites(&self) -> Result<Vec<Site>> {
        match &self.auth_method {
            AuthMethod::ApiKey(_) => {
                let url = format!("{}/proxy/network/integration/v1/sites", self.base_url);

                debug!("Making request to: {}", url);

                let mut headers = HeaderMap::new();
                headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                if let AuthMethod::ApiKey(key) = &self.auth_method {
                    headers.insert("X-API-KEY", HeaderValue::from_str(key).unwrap());
                }

                let response = self.client.get(&url).headers(headers).send().await?;

                if !response.status().is_success() {
                    return Err(anyhow!("API request failed: {}", response.status()));
                }

                let api_response: IntegrationResponse<IntegrationSite> = response.json().await?;
                Ok(api_response.data.into_iter().map(|s| s.to_site()).collect())
            }
            AuthMethod::UserPass { .. } => self.get_legacy("/self/sites").await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unifi_client_creation_with_api_key() {
        let client = UniFiClient::new(
            "https://192.168.1.1:8443".to_string(),
            Some("test-api-key".to_string()),
            None,
            None,
            "default".to_string(),
            Duration::from_secs(10),
            false,
        );
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.site, "default");
        assert_eq!(client.base_url, "https://192.168.1.1:8443");
    }

    #[test]
    fn test_unifi_client_creation_with_username_password() {
        let client = UniFiClient::new(
            "https://192.168.1.1:8443".to_string(),
            None,
            Some("admin".to_string()),
            Some("password".to_string()),
            "default".to_string(),
            Duration::from_secs(10),
            false,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_unifi_client_creation_missing_auth() {
        let client = UniFiClient::new(
            "https://192.168.1.1:8443".to_string(),
            None,
            None,
            None,
            "default".to_string(),
            Duration::from_secs(10),
            false,
        );
        assert!(client.is_err());
        let err = client.err().unwrap();
        assert!(
            err.to_string()
                .contains("Either API key or username/password must be provided")
        );
    }

    #[test]
    fn test_unifi_client_strips_trailing_slash() {
        let client = UniFiClient::new(
            "https://192.168.1.1:8443/".to_string(),
            Some("test-api-key".to_string()),
            None,
            None,
            "default".to_string(),
            Duration::from_secs(10),
            false,
        )
        .unwrap();
        assert_eq!(client.base_url, "https://192.168.1.1:8443");
    }

    #[test]
    fn test_unifi_error_display() {
        let error = UniFiError::AuthenticationFailed;
        assert_eq!(error.to_string(), "Authentication failed");

        let error = UniFiError::ParseError("Invalid JSON".to_string());
        assert_eq!(error.to_string(), "Failed to parse response: Invalid JSON");
    }

    #[test]
    fn test_device_deserialize() {
        let json = r#"{
            "_id": "device123",
            "name": "Test AP",
            "mac": "00:11:22:33:44:55",
            "type": "uap",
            "model": "UAP-AC-Pro",
            "version": "4.3.20",
            "adopted": true,
            "state": 1,
            "uptime": 86400
        }"#;
        let device: Device = serde_json::from_str(json).unwrap();
        assert_eq!(device._id, "device123");
        assert_eq!(device.name, Some("Test AP".to_string()));
        assert_eq!(device.mac, "00:11:22:33:44:55");
        assert_eq!(device.device_type, "uap");
        assert_eq!(device.model, Some("UAP-AC-Pro".to_string()));
        assert_eq!(device.version, Some("4.3.20".to_string()));
        assert_eq!(device.adopted, true);
        assert_eq!(device.state, 1);
        assert_eq!(device.uptime, Some(86400));
    }

    #[test]
    fn test_client_deserialize() {
        let json = r#"{
            "_id": "client123",
            "mac": "aa:bb:cc:dd:ee:ff",
            "ip": "192.168.1.100",
            "hostname": "test-laptop",
            "name": "Test Laptop",
            "network": "LAN",
            "vlan": 10,
            "ap_mac": "00:11:22:33:44:55",
            "signal": -65,
            "tx_bytes": 1024000,
            "rx_bytes": 2048000,
            "uptime": 3600,
            "is_wired": false,
            "is_guest": false
        }"#;
        let client: Client = serde_json::from_str(json).unwrap();
        assert_eq!(client._id, "client123");
        assert_eq!(client.mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(client.ip, Some("192.168.1.100".to_string()));
        assert_eq!(client.hostname, Some("test-laptop".to_string()));
        assert_eq!(client.name, Some("Test Laptop".to_string()));
        assert_eq!(client.network, Some("LAN".to_string()));
        assert_eq!(client.vlan, Some(10));
        assert_eq!(client.ap_mac, Some("00:11:22:33:44:55".to_string()));
        assert_eq!(client.signal, Some(-65));
        assert_eq!(client.tx_bytes, Some(1024000));
        assert_eq!(client.rx_bytes, Some(2048000));
        assert_eq!(client.uptime, Some(3600));
        assert_eq!(client.is_wired, false);
        assert_eq!(client.is_guest, false);
    }

    #[test]
    fn test_site_deserialize() {
        let json = r#"{
            "_id": "site123",
            "name": "default",
            "desc": "Default Site",
            "attr_hidden_id": "hidden123",
            "attr_no_delete": true
        }"#;
        let site: Site = serde_json::from_str(json).unwrap();
        assert_eq!(site._id, "site123");
        assert_eq!(site.name, "default");
        assert_eq!(site.desc, "Default Site");
        assert_eq!(site.attr_hidden_id, Some("hidden123".to_string()));
        assert_eq!(site.attr_no_delete, Some(true));
    }

    #[test]
    fn test_sys_stats_deserialize() {
        let json = r#"{
            "loadavg_1": 1.5,
            "loadavg_5": 1.2,
            "loadavg_15": 1.0,
            "mem_total": 1073741824,
            "mem_used": 536870912
        }"#;
        let stats: SysStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.loadavg_1, Some(1.5));
        assert_eq!(stats.loadavg_5, Some(1.2));
        assert_eq!(stats.loadavg_15, Some(1.0));
        assert_eq!(stats.mem_total, Some(1073741824));
        assert_eq!(stats.mem_used, Some(536870912));
    }

    #[test]
    fn test_device_stats_deserialize() {
        let json = r#"{
            "bytes": 3072000,
            "tx_bytes": 1024000,
            "rx_bytes": 2048000,
            "tx_packets": 1000,
            "rx_packets": 2000
        }"#;
        let stats: DeviceStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.bytes, Some(3072000));
        assert_eq!(stats.tx_bytes, Some(1024000));
        assert_eq!(stats.rx_bytes, Some(2048000));
        assert_eq!(stats.tx_packets, Some(1000));
        assert_eq!(stats.rx_packets, Some(2000));
    }

    #[tokio::test]
    async fn test_ensure_authenticated_with_api_key() {
        let client = UniFiClient::new(
            "https://192.168.1.1:8443".to_string(),
            Some("test-api-key".to_string()),
            None,
            None,
            "default".to_string(),
            Duration::from_secs(10),
            false,
        )
        .unwrap();

        // API key auth should always succeed without network call
        let result = client.ensure_authenticated().await;
        assert!(result.is_ok());
    }
}
