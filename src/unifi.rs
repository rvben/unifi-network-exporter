use anyhow::{anyhow, Result};
use cookie::Cookie;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

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
struct LoginResponse {
    meta: Meta,
    data: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
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
    pub adopted: bool,
    pub state: i32,
    pub uptime: Option<i64>,
    pub sys_stats: Option<SysStats>,
    pub stat: Option<DeviceStats>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SysStats {
    pub loadavg_1: Option<f64>,
    pub loadavg_5: Option<f64>,
    pub loadavg_15: Option<f64>,
    pub mem_used: Option<i64>,
    pub mem_total: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeviceStats {
    pub bytes: Option<i64>,
    pub tx_bytes: Option<i64>,
    pub rx_bytes: Option<i64>,
    pub tx_packets: Option<i64>,
    pub rx_packets: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub _id: String,
    pub mac: String,
    pub hostname: Option<String>,
    pub name: Option<String>,
    pub ip: Option<String>,
    pub is_wired: bool,
    pub is_guest: bool,
    pub network: Option<String>,
    pub vlan: Option<i32>,
    pub rx_bytes: Option<i64>,
    pub tx_bytes: Option<i64>,
    pub signal: Option<i32>,
    pub uptime: Option<i64>,
    pub ap_mac: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Site {
    pub _id: String,
    pub name: String,
    pub desc: String,
    pub attr_hidden_id: Option<String>,
    pub attr_no_delete: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    meta: Meta,
    data: Vec<T>,
}

pub struct UniFiClient {
    client: reqwest::Client,
    base_url: String,
    username: String,
    password: String,
    site: String,
    auth_cookies: Arc<RwLock<Option<String>>>,
}

impl UniFiClient {
    pub fn new(
        base_url: String,
        username: String,
        password: String,
        site: String,
        timeout: Duration,
        verify_ssl: bool,
    ) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .danger_accept_invalid_certs(!verify_ssl)
            .cookie_store(true)
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            username,
            password,
            site,
            auth_cookies: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn ensure_authenticated(&self) -> Result<()> {
        let cookies = self.auth_cookies.read().await;
        if cookies.is_some() {
            return Ok(());
        }
        drop(cookies);

        self.login().await
    }

    async fn login(&self) -> Result<()> {
        let login_url = format!("{}/api/login", self.base_url);
        let login_data = LoginRequest {
            username: self.username.clone(),
            password: self.password.clone(),
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
            return Err(anyhow!("No authentication cookies received"));
        }

        // Store cookies for future requests
        let cookie_string = cookies.join("; ");
        let mut auth_cookies = self.auth_cookies.write().await;
        *auth_cookies = Some(cookie_string);

        Ok(())
    }

    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
    ) -> Result<Vec<T>, UniFiError> {
        let url = format!("{}/api/s/{}/{}", self.base_url, self.site, endpoint);

        let mut headers = HeaderMap::new();
        if let Some(cookies) = &*self.auth_cookies.read().await {
            headers.insert(COOKIE, HeaderValue::from_str(cookies).unwrap());
        }

        let response = self.client.get(&url).headers(headers).send().await?;

        if response.status() == 401 {
            // Try to re-authenticate
            drop(self.auth_cookies.write().await.take());
            self.login().await.map_err(|_| UniFiError::AuthenticationFailed)?;
            
            // Retry request
            let mut headers = HeaderMap::new();
            if let Some(cookies) = &*self.auth_cookies.read().await {
                headers.insert(COOKIE, HeaderValue::from_str(cookies).unwrap());
            }
            
            let response = self.client.get(&url).headers(headers).send().await?;
            
            if !response.status().is_success() {
                return Err(UniFiError::ParseError(format!(
                    "API request failed with status: {}",
                    response.status()
                )));
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
            )))
        }
    }

    pub async fn fetch_devices(&self) -> Result<Vec<Device>, UniFiError> {
        self.make_request("stat/device").await
    }

    pub async fn fetch_clients(&self) -> Result<Vec<Client>, UniFiError> {
        self.make_request("stat/sta").await
    }

    pub async fn fetch_sites(&self) -> Result<Vec<Site>, UniFiError> {
        // Sites endpoint is at the root level, not under a specific site
        let url = format!("{}/api/self/sites", self.base_url);

        let mut headers = HeaderMap::new();
        if let Some(cookies) = &*self.auth_cookies.read().await {
            headers.insert(COOKIE, HeaderValue::from_str(cookies).unwrap());
        }

        let response = self.client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(UniFiError::ParseError(format!(
                "Sites request failed with status: {}",
                response.status()
            )));
        }

        let api_response: ApiResponse<Site> = response.json().await?;
        Ok(api_response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unifi_client_creation() {
        let client = UniFiClient::new(
            "https://192.168.1.1:8443".to_string(),
            "admin".to_string(),
            "password".to_string(),
            "default".to_string(),
            Duration::from_secs(10),
            true,
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_unifi_error_display() {
        let error = UniFiError::AuthenticationFailed;
        assert_eq!(error.to_string(), "Authentication failed");
    }
}