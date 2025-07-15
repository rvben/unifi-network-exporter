use anyhow::Result;
use prometheus::{Encoder, GaugeVec, IntCounterVec, IntGaugeVec, Opts, Registry, TextEncoder};

use crate::unifi::{Client, Device, Site};

pub struct Metrics {
    registry: Registry,
    // Device metrics
    device_info: IntGaugeVec,
    device_uptime: IntGaugeVec,
    device_adopted: IntGaugeVec,
    device_state: IntGaugeVec,
    device_cpu_usage: GaugeVec,
    device_memory_usage: GaugeVec,
    device_memory_total: IntGaugeVec,
    device_bytes_total: IntCounterVec,
    device_packets_total: IntCounterVec,

    // Client metrics
    client_info: IntGaugeVec,
    client_bytes_total: IntCounterVec,
    client_signal_strength: IntGaugeVec,
    client_uptime: IntGaugeVec,
    clients_total: IntGaugeVec,

    // Site metrics
    sites_total: IntGaugeVec,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        // Device metrics
        let device_info = IntGaugeVec::new(
            Opts::new("unifi_device_info", "UniFi device information"),
            &["id", "name", "mac", "type", "model", "version"],
        )?;
        registry.register(Box::new(device_info.clone()))?;

        let device_uptime = IntGaugeVec::new(
            Opts::new("unifi_device_uptime_seconds", "Device uptime in seconds"),
            &["id", "name", "mac"],
        )?;
        registry.register(Box::new(device_uptime.clone()))?;

        let device_adopted = IntGaugeVec::new(
            Opts::new(
                "unifi_device_adopted",
                "Device adoption status (1=adopted, 0=not adopted)",
            ),
            &["id", "name", "mac"],
        )?;
        registry.register(Box::new(device_adopted.clone()))?;

        let device_state = IntGaugeVec::new(
            Opts::new("unifi_device_state", "Device state"),
            &["id", "name", "mac"],
        )?;
        registry.register(Box::new(device_state.clone()))?;

        let device_cpu_usage = GaugeVec::new(
            Opts::new("unifi_device_cpu_usage", "Device CPU usage (load average)"),
            &["id", "name", "mac", "period"],
        )?;
        registry.register(Box::new(device_cpu_usage.clone()))?;

        let device_memory_usage = GaugeVec::new(
            Opts::new(
                "unifi_device_memory_usage_ratio",
                "Device memory usage ratio",
            ),
            &["id", "name", "mac"],
        )?;
        registry.register(Box::new(device_memory_usage.clone()))?;

        let device_memory_total = IntGaugeVec::new(
            Opts::new(
                "unifi_device_memory_total_bytes",
                "Device total memory in bytes",
            ),
            &["id", "name", "mac"],
        )?;
        registry.register(Box::new(device_memory_total.clone()))?;

        let device_bytes_total = IntCounterVec::new(
            Opts::new("unifi_device_bytes_total", "Total bytes transferred"),
            &["id", "name", "mac", "direction"],
        )?;
        registry.register(Box::new(device_bytes_total.clone()))?;

        let device_packets_total = IntCounterVec::new(
            Opts::new("unifi_device_packets_total", "Total packets transferred"),
            &["id", "name", "mac", "direction"],
        )?;
        registry.register(Box::new(device_packets_total.clone()))?;

        // Client metrics
        let client_info = IntGaugeVec::new(
            Opts::new("unifi_client_info", "UniFi client information"),
            &["id", "mac", "hostname", "name", "ip", "network", "ap_mac"],
        )?;
        registry.register(Box::new(client_info.clone()))?;

        let client_bytes_total = IntCounterVec::new(
            Opts::new(
                "unifi_client_bytes_total",
                "Total bytes transferred by client",
            ),
            &["id", "mac", "hostname", "direction"],
        )?;
        registry.register(Box::new(client_bytes_total.clone()))?;

        let client_signal_strength = IntGaugeVec::new(
            Opts::new(
                "unifi_client_signal_strength_dbm",
                "Client WiFi signal strength in dBm",
            ),
            &["id", "mac", "hostname"],
        )?;
        registry.register(Box::new(client_signal_strength.clone()))?;

        let client_uptime = IntGaugeVec::new(
            Opts::new(
                "unifi_client_uptime_seconds",
                "Client connection uptime in seconds",
            ),
            &["id", "mac", "hostname"],
        )?;
        registry.register(Box::new(client_uptime.clone()))?;

        let clients_total = IntGaugeVec::new(
            Opts::new("unifi_clients_total", "Total number of clients"),
            &["type", "network", "is_guest"],
        )?;
        registry.register(Box::new(clients_total.clone()))?;

        // Site metrics
        let sites_total =
            IntGaugeVec::new(Opts::new("unifi_sites_total", "Total number of sites"), &[])?;
        registry.register(Box::new(sites_total.clone()))?;

        Ok(Self {
            registry,
            device_info,
            device_uptime,
            device_adopted,
            device_state,
            device_cpu_usage,
            device_memory_usage,
            device_memory_total,
            device_bytes_total,
            device_packets_total,
            client_info,
            client_bytes_total,
            client_signal_strength,
            client_uptime,
            clients_total,
            sites_total,
        })
    }

    pub fn update_devices(&mut self, devices: &[Device]) {
        // Clear existing metrics
        self.device_info.reset();
        self.device_uptime.reset();
        self.device_adopted.reset();
        self.device_state.reset();
        self.device_cpu_usage.reset();
        self.device_memory_usage.reset();
        self.device_memory_total.reset();

        for device in devices {
            let name = device.name.as_deref().unwrap_or("unknown");
            let model = device.model.as_deref().unwrap_or("unknown");
            let version = device.version.as_deref().unwrap_or("unknown");

            // Device info
            let device_info_labels = vec![
                device._id.clone(),
                name.to_string(),
                device.mac.clone(),
                device.device_type.clone(),
                model.to_string(),
                version.to_string(),
            ];
            let device_info_refs: Vec<&str> =
                device_info_labels.iter().map(|s| s.as_str()).collect();
            self.device_info.with_label_values(&device_info_refs).set(1);

            // Uptime
            if let Some(uptime) = device.uptime {
                let uptime_labels = vec![device._id.clone(), name.to_string(), device.mac.clone()];
                let uptime_refs: Vec<&str> = uptime_labels.iter().map(|s| s.as_str()).collect();
                self.device_uptime
                    .with_label_values(&uptime_refs)
                    .set(uptime);
            }

            // Adoption status
            let adopted_labels = vec![device._id.clone(), name.to_string(), device.mac.clone()];
            let adopted_refs: Vec<&str> = adopted_labels.iter().map(|s| s.as_str()).collect();
            self.device_adopted
                .with_label_values(&adopted_refs)
                .set(if device.adopted { 1 } else { 0 });

            // State
            let state_labels = vec![device._id.clone(), name.to_string(), device.mac.clone()];
            let state_refs: Vec<&str> = state_labels.iter().map(|s| s.as_str()).collect();
            self.device_state
                .with_label_values(&state_refs)
                .set(device.state as i64);

            // System stats
            if let Some(sys_stats) = &device.sys_stats {
                if let Some(load1) = sys_stats.loadavg_1 {
                    let cpu1_labels = vec![
                        device._id.clone(),
                        name.to_string(),
                        device.mac.clone(),
                        "1m".to_string(),
                    ];
                    let cpu1_refs: Vec<&str> = cpu1_labels.iter().map(|s| s.as_str()).collect();
                    self.device_cpu_usage
                        .with_label_values(&cpu1_refs)
                        .set(load1);
                }
                if let Some(load5) = sys_stats.loadavg_5 {
                    let cpu5_labels = vec![
                        device._id.clone(),
                        name.to_string(),
                        device.mac.clone(),
                        "5m".to_string(),
                    ];
                    let cpu5_refs: Vec<&str> = cpu5_labels.iter().map(|s| s.as_str()).collect();
                    self.device_cpu_usage
                        .with_label_values(&cpu5_refs)
                        .set(load5);
                }
                if let Some(load15) = sys_stats.loadavg_15 {
                    let cpu15_labels = vec![
                        device._id.clone(),
                        name.to_string(),
                        device.mac.clone(),
                        "15m".to_string(),
                    ];
                    let cpu15_refs: Vec<&str> = cpu15_labels.iter().map(|s| s.as_str()).collect();
                    self.device_cpu_usage
                        .with_label_values(&cpu15_refs)
                        .set(load15);
                }

                if let (Some(mem_used), Some(mem_total)) = (sys_stats.mem_used, sys_stats.mem_total)
                {
                    if mem_total > 0 {
                        let usage_ratio = mem_used as f64 / mem_total as f64;
                        let mem_usage_labels =
                            vec![device._id.clone(), name.to_string(), device.mac.clone()];
                        let mem_usage_refs: Vec<&str> =
                            mem_usage_labels.iter().map(|s| s.as_str()).collect();
                        self.device_memory_usage
                            .with_label_values(&mem_usage_refs)
                            .set(usage_ratio);
                    }
                    let mem_total_labels =
                        vec![device._id.clone(), name.to_string(), device.mac.clone()];
                    let mem_total_refs: Vec<&str> =
                        mem_total_labels.iter().map(|s| s.as_str()).collect();
                    self.device_memory_total
                        .with_label_values(&mem_total_refs)
                        .set(mem_total);
                }
            }

            // Traffic stats
            if let Some(stats) = &device.stat {
                if let Some(tx_bytes) = stats.tx_bytes {
                    let tx_bytes_labels = vec![
                        device._id.clone(),
                        name.to_string(),
                        device.mac.clone(),
                        "tx".to_string(),
                    ];
                    let tx_bytes_refs: Vec<&str> =
                        tx_bytes_labels.iter().map(|s| s.as_str()).collect();
                    self.device_bytes_total
                        .with_label_values(&tx_bytes_refs)
                        .inc_by(tx_bytes as u64);
                }
                if let Some(rx_bytes) = stats.rx_bytes {
                    let rx_bytes_labels = vec![
                        device._id.clone(),
                        name.to_string(),
                        device.mac.clone(),
                        "rx".to_string(),
                    ];
                    let rx_bytes_refs: Vec<&str> =
                        rx_bytes_labels.iter().map(|s| s.as_str()).collect();
                    self.device_bytes_total
                        .with_label_values(&rx_bytes_refs)
                        .inc_by(rx_bytes as u64);
                }
                if let Some(tx_packets) = stats.tx_packets {
                    let tx_packets_labels = vec![
                        device._id.clone(),
                        name.to_string(),
                        device.mac.clone(),
                        "tx".to_string(),
                    ];
                    let tx_packets_refs: Vec<&str> =
                        tx_packets_labels.iter().map(|s| s.as_str()).collect();
                    self.device_packets_total
                        .with_label_values(&tx_packets_refs)
                        .inc_by(tx_packets as u64);
                }
                if let Some(rx_packets) = stats.rx_packets {
                    let rx_packets_labels = vec![
                        device._id.clone(),
                        name.to_string(),
                        device.mac.clone(),
                        "rx".to_string(),
                    ];
                    let rx_packets_refs: Vec<&str> =
                        rx_packets_labels.iter().map(|s| s.as_str()).collect();
                    self.device_packets_total
                        .with_label_values(&rx_packets_refs)
                        .inc_by(rx_packets as u64);
                }
            }
        }
    }

    pub fn update_clients(&mut self, clients: &[Client]) {
        // Clear existing metrics
        self.client_info.reset();
        self.client_signal_strength.reset();
        self.client_uptime.reset();
        self.clients_total.reset();

        // Count clients by type
        let mut wired_count = 0;
        let mut wireless_count = 0;
        let mut guest_count = 0;
        let mut network_counts: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();

        for client in clients {
            let hostname = client.hostname.as_deref().unwrap_or("");
            let name = client.name.as_deref().unwrap_or("");
            let ip = client.ip.as_deref().unwrap_or("");
            let network = client.network.as_deref().unwrap_or("unknown");
            let ap_mac = client.ap_mac.as_deref().unwrap_or("");

            // Client info
            let client_info_labels = vec![
                client._id.clone(),
                client.mac.clone(),
                hostname.to_string(),
                name.to_string(),
                ip.to_string(),
                network.to_string(),
                ap_mac.to_string(),
            ];
            let client_info_refs: Vec<&str> =
                client_info_labels.iter().map(|s| s.as_str()).collect();
            self.client_info.with_label_values(&client_info_refs).set(1);

            // Traffic
            if let Some(tx_bytes) = client.tx_bytes {
                let tx_labels = vec![
                    client._id.clone(),
                    client.mac.clone(),
                    hostname.to_string(),
                    "tx".to_string(),
                ];
                let tx_refs: Vec<&str> = tx_labels.iter().map(|s| s.as_str()).collect();
                self.client_bytes_total
                    .with_label_values(&tx_refs)
                    .inc_by(tx_bytes as u64);
            }
            if let Some(rx_bytes) = client.rx_bytes {
                let rx_labels = vec![
                    client._id.clone(),
                    client.mac.clone(),
                    hostname.to_string(),
                    "rx".to_string(),
                ];
                let rx_refs: Vec<&str> = rx_labels.iter().map(|s| s.as_str()).collect();
                self.client_bytes_total
                    .with_label_values(&rx_refs)
                    .inc_by(rx_bytes as u64);
            }

            // Signal strength (wireless only)
            if !client.is_wired {
                if let Some(signal) = client.signal {
                    let signal_labels =
                        vec![client._id.clone(), client.mac.clone(), hostname.to_string()];
                    let signal_refs: Vec<&str> = signal_labels.iter().map(|s| s.as_str()).collect();
                    self.client_signal_strength
                        .with_label_values(&signal_refs)
                        .set(signal as i64);
                }
            }

            // Uptime
            if let Some(uptime) = client.uptime {
                let uptime_labels =
                    vec![client._id.clone(), client.mac.clone(), hostname.to_string()];
                let uptime_refs: Vec<&str> = uptime_labels.iter().map(|s| s.as_str()).collect();
                self.client_uptime
                    .with_label_values(&uptime_refs)
                    .set(uptime);
            }

            // Count clients
            if client.is_wired {
                wired_count += 1;
            } else {
                wireless_count += 1;
            }
            if client.is_guest {
                guest_count += 1;
            }
            *network_counts.entry(network.to_string()).or_insert(0) += 1;
        }

        // Update totals
        let wired_labels = vec!["wired".to_string(), "all".to_string(), "false".to_string()];
        let wired_refs: Vec<&str> = wired_labels.iter().map(|s| s.as_str()).collect();
        self.clients_total
            .with_label_values(&wired_refs)
            .set(wired_count);
        let wireless_labels = vec![
            "wireless".to_string(),
            "all".to_string(),
            "false".to_string(),
        ];
        let wireless_refs: Vec<&str> = wireless_labels.iter().map(|s| s.as_str()).collect();
        self.clients_total
            .with_label_values(&wireless_refs)
            .set(wireless_count);
        let guest_labels = vec!["all".to_string(), "all".to_string(), "true".to_string()];
        let guest_refs: Vec<&str> = guest_labels.iter().map(|s| s.as_str()).collect();
        self.clients_total
            .with_label_values(&guest_refs)
            .set(guest_count);
        let all_labels = vec!["all".to_string(), "all".to_string(), "false".to_string()];
        let all_refs: Vec<&str> = all_labels.iter().map(|s| s.as_str()).collect();
        self.clients_total
            .with_label_values(&all_refs)
            .set((wired_count + wireless_count - guest_count).max(0));

        // Per-network counts
        for (network, count) in network_counts {
            let network_labels = vec!["all".to_string(), network.clone(), "all".to_string()];
            let network_refs: Vec<&str> = network_labels.iter().map(|s| s.as_str()).collect();
            self.clients_total
                .with_label_values(&network_refs)
                .set(count);
        }
    }

    pub fn update_sites(&mut self, sites: &[Site]) {
        self.sites_total.reset();
        let empty_labels: &[&str] = &[];
        self.sites_total
            .with_label_values(empty_labels)
            .set(sites.len() as i64);
    }

    pub fn gather(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unifi::{Device, Client, Site, SysStats, DeviceStats};

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_metrics_gather() {
        let mut metrics = Metrics::new().unwrap();
        
        // First gather might be empty
        let initial_output = metrics.gather();
        
        // Update with some data to ensure metrics are populated
        let devices = vec![Device {
            _id: "test".to_string(),
            name: Some("Test Device".to_string()),
            mac: "00:00:00:00:00:00".to_string(),
            device_type: "uap".to_string(),
            model: Some("Test Model".to_string()),
            version: Some("1.0".to_string()),
            adopted: true,
            state: 1,
            uptime: Some(100),
            sys_stats: None,
            stat: None,
        }];
        
        metrics.update_devices(&devices);
        let output = metrics.gather();
        
        // Now we should have output
        assert!(!output.is_empty() || !initial_output.is_empty());
        if !output.is_empty() {
            assert!(output.contains("# HELP") || output.contains("unifi_"));
        }
    }

    #[test]
    fn test_update_devices() {
        let mut metrics = Metrics::new().unwrap();
        let devices = vec![
            Device {
                _id: "device1".to_string(),
                name: Some("Test AP".to_string()),
                mac: "00:11:22:33:44:55".to_string(),
                device_type: "uap".to_string(),
                model: Some("UAP-AC-Pro".to_string()),
                version: Some("4.3.20".to_string()),
                adopted: true,
                state: 1,
                uptime: Some(86400),
                sys_stats: Some(SysStats {
                    loadavg_1: Some(1.5),
                    loadavg_5: Some(1.2),
                    loadavg_15: Some(1.0),
                    mem_total: Some(1073741824),
                    mem_used: Some(536870912),
                }),
                stat: Some(DeviceStats {
                    bytes: Some(3072000),
                    tx_bytes: Some(1024000),
                    rx_bytes: Some(2048000),
                    tx_packets: Some(1000),
                    rx_packets: Some(2000),
                }),
            },
            Device {
                _id: "device2".to_string(),
                name: None,
                mac: "00:11:22:33:44:66".to_string(),
                device_type: "usw".to_string(),
                model: None,
                version: None,
                adopted: false,
                state: 0,
                uptime: None,
                sys_stats: None,
                stat: None,
            },
        ];
        
        metrics.update_devices(&devices);
        let output = metrics.gather();
        
        // Check device info metric
        assert!(output.contains("unifi_device_info"));
        assert!(output.contains("Test AP"));
        assert!(output.contains("UAP-AC-Pro"));
        assert!(output.contains("4.3.20"));
        
        // Check uptime metric
        assert!(output.contains("unifi_device_uptime_seconds"));
        assert!(output.contains("86400"));
        
        // Check adoption status
        assert!(output.contains("unifi_device_adopted"));
        
        // Check CPU usage
        assert!(output.contains("unifi_device_cpu_usage"));
        assert!(output.contains("1.5"));
        
        // Check memory usage
        assert!(output.contains("unifi_device_memory_usage_ratio"));
        assert!(output.contains("0.5")); // 536870912 / 1073741824 = 0.5
        
        // Check bytes and packets
        assert!(output.contains("unifi_device_bytes_total"));
        assert!(output.contains("unifi_device_packets_total"));
    }

    #[test]
    fn test_update_clients() {
        let mut metrics = Metrics::new().unwrap();
        let clients = vec![
            Client {
                _id: "client1".to_string(),
                mac: "aa:bb:cc:dd:ee:ff".to_string(),
                ip: Some("192.168.1.100".to_string()),
                hostname: Some("test-laptop".to_string()),
                name: Some("Test Laptop".to_string()),
                network: Some("LAN".to_string()),
                vlan: Some(10),
                ap_mac: Some("00:11:22:33:44:55".to_string()),
                signal: Some(-65),
                tx_bytes: Some(1024000),
                rx_bytes: Some(2048000),
                uptime: Some(3600),
                is_wired: false,
                is_guest: false,
            },
            Client {
                _id: "client2".to_string(),
                mac: "aa:bb:cc:dd:ee:00".to_string(),
                ip: Some("192.168.1.101".to_string()),
                hostname: None,
                name: None,
                network: Some("Guest".to_string()),
                vlan: None,
                ap_mac: None,
                signal: None,
                tx_bytes: Some(512000),
                rx_bytes: Some(1024000),
                uptime: Some(1800),
                is_wired: true,
                is_guest: true,
            },
        ];
        
        metrics.update_clients(&clients);
        let output = metrics.gather();
        
        // Check client info metric
        assert!(output.contains("unifi_client_info"));
        assert!(output.contains("test-laptop"));
        assert!(output.contains("Test Laptop"));
        assert!(output.contains("192.168.1.100"));
        
        // Check signal strength (only for wireless)
        assert!(output.contains("unifi_client_signal_strength_dbm"));
        assert!(output.contains("-65"));
        
        // Check uptime
        assert!(output.contains("unifi_client_uptime_seconds"));
        assert!(output.contains("3600"));
        
        // Check client counts
        assert!(output.contains("unifi_clients_total"));
        assert!(output.contains(r#"type="wireless"#));
        assert!(output.contains(r#"type="wired"#));
        assert!(output.contains(r#"is_guest="true"#));
        assert!(output.contains(r#"network="LAN"#));
        assert!(output.contains(r#"network="Guest"#));
    }

    #[test]
    fn test_update_sites() {
        let mut metrics = Metrics::new().unwrap();
        let sites = vec![
            Site {
                _id: "site1".to_string(),
                name: "default".to_string(),
                desc: "Default Site".to_string(),
                attr_hidden_id: None,
                attr_no_delete: Some(true),
            },
            Site {
                _id: "site2".to_string(),
                name: "branch".to_string(),
                desc: "Branch Office".to_string(),
                attr_hidden_id: Some("hidden".to_string()),
                attr_no_delete: Some(false),
            },
        ];
        
        metrics.update_sites(&sites);
        let output = metrics.gather();
        
        // Check sites total metric
        assert!(output.contains("unifi_sites_total"));
        assert!(output.contains("2")); // 2 sites
    }

    #[test]
    fn test_update_devices_with_missing_data() {
        let mut metrics = Metrics::new().unwrap();
        let devices = vec![Device {
            _id: "device1".to_string(),
            name: None,
            mac: "00:11:22:33:44:55".to_string(),
            device_type: "uap".to_string(),
            model: None,
            version: None,
            adopted: true,
            state: 1,
            uptime: None,
            sys_stats: None,
            stat: None,
        }];
        
        metrics.update_devices(&devices);
        let output = metrics.gather();
        
        // Should handle missing values gracefully
        assert!(output.contains("unifi_device_info"));
        assert!(output.contains("unknown")); // Default for missing name/model/version
    }

    #[test]
    fn test_memory_usage_calculation() {
        let mut metrics = Metrics::new().unwrap();
        let devices = vec![Device {
            _id: "device1".to_string(),
            name: Some("Test".to_string()),
            mac: "00:11:22:33:44:55".to_string(),
            device_type: "uap".to_string(),
            model: Some("Model".to_string()),
            version: Some("1.0".to_string()),
            adopted: true,
            state: 1,
            uptime: None,
            sys_stats: Some(SysStats {
                loadavg_1: None,
                loadavg_5: None,
                loadavg_15: None,
                mem_total: Some(1000),
                mem_used: Some(750),
            }),
            stat: None,
        }];
        
        metrics.update_devices(&devices);
        let output = metrics.gather();
        
        // Should calculate memory usage ratio correctly
        assert!(output.contains("unifi_device_memory_usage_ratio"));
        assert!(output.contains("0.75")); // 750/1000 = 0.75
    }

    #[test]
    fn test_client_counts() {
        let mut metrics = Metrics::new().unwrap();
        let clients = vec![
            // Wired non-guest client
            Client {
                _id: "c1".to_string(),
                mac: "00:00:00:00:00:01".to_string(),
                ip: None,
                hostname: None,
                name: None,
                network: Some("LAN".to_string()),
                vlan: None,
                ap_mac: None,
                signal: None,
                tx_bytes: None,
                rx_bytes: None,
                uptime: None,
                is_wired: true,
                is_guest: false,
            },
            // Wireless guest client
            Client {
                _id: "c2".to_string(),
                mac: "00:00:00:00:00:02".to_string(),
                ip: None,
                hostname: None,
                name: None,
                network: Some("Guest".to_string()),
                vlan: None,
                ap_mac: None,
                signal: None,
                tx_bytes: None,
                rx_bytes: None,
                uptime: None,
                is_wired: false,
                is_guest: true,
            },
            // Another wireless non-guest client
            Client {
                _id: "c3".to_string(),
                mac: "00:00:00:00:00:03".to_string(),
                ip: None,
                hostname: None,
                name: None,
                network: Some("LAN".to_string()),
                vlan: None,
                ap_mac: None,
                signal: None,
                tx_bytes: None,
                rx_bytes: None,
                uptime: None,
                is_wired: false,
                is_guest: false,
            },
        ];
        
        metrics.update_clients(&clients);
        let output = metrics.gather();
        
        // Verify counts are correct
        assert!(output.contains(r#"unifi_clients_total{is_guest="false",network="all",type="wired"} 1"#));
        assert!(output.contains(r#"unifi_clients_total{is_guest="false",network="all",type="wireless"} 2"#));
        assert!(output.contains(r#"unifi_clients_total{is_guest="true",network="all",type="all"} 1"#));
    }
}
