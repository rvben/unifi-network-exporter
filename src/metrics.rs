use anyhow::Result;
use prometheus::{
    Encoder, GaugeVec, IntCounterVec, IntGaugeVec, Opts, Registry, TextEncoder,
};

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
            Opts::new("unifi_device_adopted", "Device adoption status (1=adopted, 0=not adopted)"),
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
            Opts::new("unifi_device_memory_usage_ratio", "Device memory usage ratio"),
            &["id", "name", "mac"],
        )?;
        registry.register(Box::new(device_memory_usage.clone()))?;

        let device_memory_total = IntGaugeVec::new(
            Opts::new("unifi_device_memory_total_bytes", "Device total memory in bytes"),
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
            Opts::new("unifi_client_bytes_total", "Total bytes transferred by client"),
            &["id", "mac", "hostname", "direction"],
        )?;
        registry.register(Box::new(client_bytes_total.clone()))?;

        let client_signal_strength = IntGaugeVec::new(
            Opts::new("unifi_client_signal_strength_dbm", "Client WiFi signal strength in dBm"),
            &["id", "mac", "hostname"],
        )?;
        registry.register(Box::new(client_signal_strength.clone()))?;

        let client_uptime = IntGaugeVec::new(
            Opts::new("unifi_client_uptime_seconds", "Client connection uptime in seconds"),
            &["id", "mac", "hostname"],
        )?;
        registry.register(Box::new(client_uptime.clone()))?;

        let clients_total = IntGaugeVec::new(
            Opts::new("unifi_clients_total", "Total number of clients"),
            &["type", "network", "is_guest"],
        )?;
        registry.register(Box::new(clients_total.clone()))?;

        // Site metrics
        let sites_total = IntGaugeVec::new(
            Opts::new("unifi_sites_total", "Total number of sites"),
            &[],
        )?;
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
            self.device_info
                .with_label_values(&[
                    &device._id,
                    name,
                    &device.mac,
                    &device.device_type,
                    model,
                    version,
                ])
                .set(1);

            // Uptime
            if let Some(uptime) = device.uptime {
                self.device_uptime
                    .with_label_values(&[&device._id, name, &device.mac])
                    .set(uptime);
            }

            // Adoption status
            self.device_adopted
                .with_label_values(&[&device._id, name, &device.mac])
                .set(if device.adopted { 1 } else { 0 });

            // State
            self.device_state
                .with_label_values(&[&device._id, name, &device.mac])
                .set(device.state as i64);

            // System stats
            if let Some(sys_stats) = &device.sys_stats {
                if let Some(load1) = sys_stats.loadavg_1 {
                    self.device_cpu_usage
                        .with_label_values(&[&device._id, name, &device.mac, "1m"])
                        .set(load1);
                }
                if let Some(load5) = sys_stats.loadavg_5 {
                    self.device_cpu_usage
                        .with_label_values(&[&device._id, name, &device.mac, "5m"])
                        .set(load5);
                }
                if let Some(load15) = sys_stats.loadavg_15 {
                    self.device_cpu_usage
                        .with_label_values(&[&device._id, name, &device.mac, "15m"])
                        .set(load15);
                }

                if let (Some(mem_used), Some(mem_total)) = (sys_stats.mem_used, sys_stats.mem_total) {
                    if mem_total > 0 {
                        let usage_ratio = mem_used as f64 / mem_total as f64;
                        self.device_memory_usage
                            .with_label_values(&[&device._id, name, &device.mac])
                            .set(usage_ratio);
                    }
                    self.device_memory_total
                        .with_label_values(&[&device._id, name, &device.mac])
                        .set(mem_total);
                }
            }

            // Traffic stats
            if let Some(stats) = &device.stat {
                if let Some(tx_bytes) = stats.tx_bytes {
                    self.device_bytes_total
                        .with_label_values(&[&device._id, name, &device.mac, "tx"])
                        .inc_by(tx_bytes as u64);
                }
                if let Some(rx_bytes) = stats.rx_bytes {
                    self.device_bytes_total
                        .with_label_values(&[&device._id, name, &device.mac, "rx"])
                        .inc_by(rx_bytes as u64);
                }
                if let Some(tx_packets) = stats.tx_packets {
                    self.device_packets_total
                        .with_label_values(&[&device._id, name, &device.mac, "tx"])
                        .inc_by(tx_packets as u64);
                }
                if let Some(rx_packets) = stats.rx_packets {
                    self.device_packets_total
                        .with_label_values(&[&device._id, name, &device.mac, "rx"])
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
        let mut network_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

        for client in clients {
            let hostname = client.hostname.as_deref().unwrap_or("");
            let name = client.name.as_deref().unwrap_or("");
            let ip = client.ip.as_deref().unwrap_or("");
            let network = client.network.as_deref().unwrap_or("unknown");
            let ap_mac = client.ap_mac.as_deref().unwrap_or("");

            // Client info
            self.client_info
                .with_label_values(&[
                    &client._id,
                    &client.mac,
                    hostname,
                    name,
                    ip,
                    network,
                    ap_mac,
                ])
                .set(1);

            // Traffic
            if let Some(tx_bytes) = client.tx_bytes {
                self.client_bytes_total
                    .with_label_values(&[&client._id, &client.mac, hostname, "tx"])
                    .inc_by(tx_bytes as u64);
            }
            if let Some(rx_bytes) = client.rx_bytes {
                self.client_bytes_total
                    .with_label_values(&[&client._id, &client.mac, hostname, "rx"])
                    .inc_by(rx_bytes as u64);
            }

            // Signal strength (wireless only)
            if !client.is_wired {
                if let Some(signal) = client.signal {
                    self.client_signal_strength
                        .with_label_values(&[&client._id, &client.mac, hostname])
                        .set(signal as i64);
                }
            }

            // Uptime
            if let Some(uptime) = client.uptime {
                self.client_uptime
                    .with_label_values(&[&client._id, &client.mac, hostname])
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
        self.clients_total
            .with_label_values(&["wired", "all", "false"])
            .set(wired_count);
        self.clients_total
            .with_label_values(&["wireless", "all", "false"])
            .set(wireless_count);
        self.clients_total
            .with_label_values(&["all", "all", "true"])
            .set(guest_count);
        self.clients_total
            .with_label_values(&["all", "all", "false"])
            .set((wired_count + wireless_count - guest_count).max(0));

        // Per-network counts
        for (network, count) in network_counts {
            self.clients_total
                .with_label_values(&["all", &network, "all"])
                .set(count);
        }
    }

    pub fn update_sites(&mut self, sites: &[Site]) {
        self.sites_total.reset();
        self.sites_total
            .with_label_values(&[])
            .set(sites.len() as i64);
    }

    pub fn gather(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_metrics_gather() {
        let metrics = Metrics::new().unwrap();
        let output = metrics.gather();
        assert!(!output.is_empty());
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
    }
}