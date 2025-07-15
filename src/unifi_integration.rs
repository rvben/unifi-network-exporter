use serde::Deserialize;

// Integration API response wrapper
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct IntegrationResponse<T> {
    pub offset: u32,
    pub limit: u32,
    pub count: u32,
    #[serde(rename = "totalCount")]
    pub total_count: u32,
    pub data: Vec<T>,
}

// Site structure for Integration API
#[derive(Debug, Deserialize, Clone)]
pub struct IntegrationSite {
    pub id: String,
    #[serde(rename = "internalReference")]
    pub internal_reference: String,
    pub name: String,
}

// Device structure for Integration API
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct IntegrationDevice {
    pub id: String,
    pub name: Option<String>,
    pub model: String,
    #[serde(rename = "macAddress")]
    pub mac_address: String,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    pub state: String,
    pub features: Vec<String>,
    pub interfaces: Vec<String>,
}

// Client structure for Integration API
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct IntegrationClient {
    #[serde(rename = "type")]
    pub client_type: String,
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "connectedAt")]
    pub connected_at: String,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(rename = "macAddress")]
    pub mac_address: String,
    #[serde(rename = "uplinkDeviceId")]
    pub uplink_device_id: Option<String>,
    pub access: ClientAccess,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClientAccess {
    #[serde(rename = "type")]
    pub access_type: String,
}

// Conversion functions to map to our existing structures
impl IntegrationDevice {
    pub fn to_device(&self) -> crate::unifi::Device {
        crate::unifi::Device {
            _id: self.id.clone(),
            name: self.name.clone(),
            mac: self.mac_address.clone(),
            device_type: self.features.first().unwrap_or(&"unknown".to_string()).clone(),
            model: Some(self.model.clone()),
            version: None,
            adopted: self.state == "ONLINE",
            state: if self.state == "ONLINE" { 1 } else { 0 },
            uptime: None,
            sys_stats: None,
            stat: None,
        }
    }
}

impl IntegrationClient {
    pub fn to_client(&self) -> crate::unifi::Client {
        crate::unifi::Client {
            _id: self.id.clone(),
            mac: self.mac_address.clone(),
            ip: self.ip_address.clone(),
            hostname: self.name.clone(),
            name: self.name.clone(),
            network: None,
            vlan: None,
            ap_mac: self.uplink_device_id.clone(),
            signal: None,
            tx_bytes: None,
            rx_bytes: None,
            uptime: None,
            is_wired: self.client_type == "WIRED",
            is_guest: self.access.access_type == "GUEST",
        }
    }
}

impl IntegrationSite {
    pub fn to_site(&self) -> crate::unifi::Site {
        crate::unifi::Site {
            _id: self.id.clone(),
            name: self.internal_reference.clone(),
            desc: self.name.clone(),
            attr_hidden_id: None,
            attr_no_delete: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_response_deserialize() {
        let json = r#"{
            "offset": 0,
            "limit": 50,
            "count": 2,
            "totalCount": 2,
            "data": []
        }"#;
        let response: IntegrationResponse<IntegrationDevice> = serde_json::from_str(json).unwrap();
        assert_eq!(response.offset, 0);
        assert_eq!(response.limit, 50);
        assert_eq!(response.count, 2);
        assert_eq!(response.total_count, 2);
        assert_eq!(response.data.len(), 0);
    }

    #[test]
    fn test_integration_site_deserialize() {
        let json = r#"{
            "id": "88f7af54-98f8-306a-a1c7-c9349722b1f6",
            "internalReference": "default",
            "name": "Default Site"
        }"#;
        let site: IntegrationSite = serde_json::from_str(json).unwrap();
        assert_eq!(site.id, "88f7af54-98f8-306a-a1c7-c9349722b1f6");
        assert_eq!(site.internal_reference, "default");
        assert_eq!(site.name, "Default Site");
    }

    #[test]
    fn test_integration_device_deserialize() {
        let json = r#"{
            "id": "device123",
            "name": "Test AP",
            "model": "UAP-AC-Pro",
            "macAddress": "00:11:22:33:44:55",
            "ipAddress": "192.168.1.10",
            "state": "ONLINE",
            "features": ["uap", "wireless"],
            "interfaces": ["eth0", "wlan0"]
        }"#;
        let device: IntegrationDevice = serde_json::from_str(json).unwrap();
        assert_eq!(device.id, "device123");
        assert_eq!(device.name, Some("Test AP".to_string()));
        assert_eq!(device.model, "UAP-AC-Pro");
        assert_eq!(device.mac_address, "00:11:22:33:44:55");
        assert_eq!(device.ip_address, Some("192.168.1.10".to_string()));
        assert_eq!(device.state, "ONLINE");
        assert_eq!(device.features, vec!["uap", "wireless"]);
        assert_eq!(device.interfaces, vec!["eth0", "wlan0"]);
    }

    #[test]
    fn test_integration_client_deserialize() {
        let json = r#"{
            "type": "WIRED",
            "id": "client123",
            "name": "Test Client",
            "connectedAt": "2023-01-01T12:00:00Z",
            "ipAddress": "192.168.1.100",
            "macAddress": "aa:bb:cc:dd:ee:ff",
            "uplinkDeviceId": "device123",
            "access": {
                "type": "NORMAL"
            }
        }"#;
        let client: IntegrationClient = serde_json::from_str(json).unwrap();
        assert_eq!(client.client_type, "WIRED");
        assert_eq!(client.id, "client123");
        assert_eq!(client.name, Some("Test Client".to_string()));
        assert_eq!(client.connected_at, "2023-01-01T12:00:00Z");
        assert_eq!(client.ip_address, Some("192.168.1.100".to_string()));
        assert_eq!(client.mac_address, "aa:bb:cc:dd:ee:ff");
        assert_eq!(client.uplink_device_id, Some("device123".to_string()));
        assert_eq!(client.access.access_type, "NORMAL");
    }

    #[test]
    fn test_integration_device_to_device() {
        let int_device = IntegrationDevice {
            id: "device123".to_string(),
            name: Some("Test AP".to_string()),
            model: "UAP-AC-Pro".to_string(),
            mac_address: "00:11:22:33:44:55".to_string(),
            ip_address: Some("192.168.1.10".to_string()),
            state: "ONLINE".to_string(),
            features: vec!["uap".to_string(), "wireless".to_string()],
            interfaces: vec!["eth0".to_string(), "wlan0".to_string()],
        };
        
        let device = int_device.to_device();
        assert_eq!(device._id, "device123");
        assert_eq!(device.name, Some("Test AP".to_string()));
        assert_eq!(device.mac, "00:11:22:33:44:55");
        assert_eq!(device.device_type, "uap");
        assert_eq!(device.model, Some("UAP-AC-Pro".to_string()));
        assert_eq!(device.version, None);
        assert_eq!(device.adopted, true);
        assert_eq!(device.state, 1);
        assert_eq!(device.uptime, None);
        assert!(device.sys_stats.is_none());
        assert!(device.stat.is_none());
    }

    #[test]
    fn test_integration_device_to_device_offline() {
        let int_device = IntegrationDevice {
            id: "device123".to_string(),
            name: None,
            model: "USW-24".to_string(),
            mac_address: "00:11:22:33:44:66".to_string(),
            ip_address: None,
            state: "OFFLINE".to_string(),
            features: vec![],
            interfaces: vec![],
        };
        
        let device = int_device.to_device();
        assert_eq!(device.name, None);
        assert_eq!(device.device_type, "unknown");
        assert_eq!(device.adopted, false);
        assert_eq!(device.state, 0);
    }

    #[test]
    fn test_integration_client_to_client_wired() {
        let int_client = IntegrationClient {
            client_type: "WIRED".to_string(),
            id: "client123".to_string(),
            name: Some("Test Client".to_string()),
            connected_at: "2023-01-01T12:00:00Z".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
            mac_address: "aa:bb:cc:dd:ee:ff".to_string(),
            uplink_device_id: Some("device123".to_string()),
            access: ClientAccess {
                access_type: "NORMAL".to_string(),
            },
        };
        
        let client = int_client.to_client();
        assert_eq!(client._id, "client123");
        assert_eq!(client.mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(client.ip, Some("192.168.1.100".to_string()));
        assert_eq!(client.hostname, Some("Test Client".to_string()));
        assert_eq!(client.name, Some("Test Client".to_string()));
        assert_eq!(client.network, None);
        assert_eq!(client.vlan, None);
        assert_eq!(client.ap_mac, Some("device123".to_string()));
        assert_eq!(client.signal, None);
        assert_eq!(client.tx_bytes, None);
        assert_eq!(client.rx_bytes, None);
        assert_eq!(client.uptime, None);
        assert_eq!(client.is_wired, true);
        assert_eq!(client.is_guest, false);
    }

    #[test]
    fn test_integration_client_to_client_wireless_guest() {
        let int_client = IntegrationClient {
            client_type: "WIRELESS".to_string(),
            id: "client456".to_string(),
            name: None,
            connected_at: "2023-01-01T12:00:00Z".to_string(),
            ip_address: None,
            mac_address: "aa:bb:cc:dd:ee:00".to_string(),
            uplink_device_id: None,
            access: ClientAccess {
                access_type: "GUEST".to_string(),
            },
        };
        
        let client = int_client.to_client();
        assert_eq!(client.is_wired, false);
        assert_eq!(client.is_guest, true);
        assert_eq!(client.hostname, None);
        assert_eq!(client.name, None);
    }

    #[test]
    fn test_integration_site_to_site() {
        let int_site = IntegrationSite {
            id: "88f7af54-98f8-306a-a1c7-c9349722b1f6".to_string(),
            internal_reference: "default".to_string(),
            name: "Default Site".to_string(),
        };
        
        let site = int_site.to_site();
        assert_eq!(site._id, "88f7af54-98f8-306a-a1c7-c9349722b1f6");
        assert_eq!(site.name, "default");
        assert_eq!(site.desc, "Default Site");
        assert_eq!(site.attr_hidden_id, None);
        assert_eq!(site.attr_no_delete, None);
    }
}