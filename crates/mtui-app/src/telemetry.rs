//! Live dashboard telemetry: WAN pick, rates, CPU/memory/firewall history.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use mtui_routeros::Resource;
use mtui_ui::{FirewallRuleMetric, TrafficSample};

const TRAFFIC_HISTORY: usize = 120;
const CPU_HISTORY: usize = 120;
const MEMORY_HISTORY: usize = 120;
const FIREWALL_HISTORY: usize = 60;

#[derive(Debug, Clone, Copy, Default)]
struct FirewallCounter {
    packets: u64,
    bytes: u64,
}

/// Mutable dashboard telemetry buffers (no UI).
#[derive(Debug, Default)]
pub struct DashboardTelemetry {
    pub traffic_interface: String,
    pub traffic_has_base: bool,
    pub traffic_rx_rate: f64,
    pub traffic_tx_rate: f64,
    pub traffic_rx_bytes: u64,
    pub traffic_tx_bytes: u64,
    pub traffic_updated: Option<Instant>,
    pub traffic_samples: Vec<TrafficSample>,
    pub cpu_core_order: Vec<String>,
    pub cpu_core_loads: HashMap<String, f64>,
    pub cpu_core_samples: HashMap<String, Vec<f64>>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub memory_samples: Vec<f64>,
    pub firewall_rules: Vec<FirewallRuleMetric>,
    pub firewall_offset: usize,
    firewall_has_base: bool,
    firewall_previous: HashMap<String, FirewallCounter>,
}

impl DashboardTelemetry {
    /// Reset rate baselines when entering the dashboard; keep sample history.
    pub fn activate(&mut self) {
        self.traffic_rx_rate = 0.0;
        self.traffic_tx_rate = 0.0;
        self.traffic_updated = None;
        self.traffic_has_base = false;
        self.firewall_has_base = false;
        self.firewall_previous.clear();
        self.firewall_offset = 0;
    }

    pub fn update_system(&mut self, resource: &Resource) {
        let total = resource_counter(resource, &["total-memory"]);
        let free = resource_counter(resource, &["free-memory"]);
        if total == 0 || free > total {
            return;
        }
        self.memory_total_bytes = total;
        self.memory_used_bytes = total - free;
        let used_percent = memory_percent(self.memory_used_bytes, total);
        self.memory_samples = append_bounded(&self.memory_samples, used_percent, MEMORY_HISTORY);
    }

    pub fn update_cpu(&mut self, cores: &[Resource], system: Option<&Resource>) {
        let mut seen = HashSet::new();
        for (index, core) in cores.iter().enumerate() {
            let numbered = format!("cpu{index}");
            let named = field(core, "name", &numbered);
            let name = field(core, "cpu", &named);
            let Some(load) = parse_f64(core.field("load").unwrap_or("")) else {
                continue;
            };
            seen.insert(name.clone());
            self.cpu_core_loads.insert(name.clone(), load);
            let samples = self.cpu_core_samples.entry(name).or_default();
            *samples = append_bounded(samples, load, CPU_HISTORY);
        }
        if seen.is_empty()
            && let Some(system) = system
            && let Some(load) = parse_f64(system.field("cpu-load").unwrap_or(""))
        {
            seen.insert("cpu".into());
            self.cpu_core_loads.insert("cpu".into(), load);
            let samples = self.cpu_core_samples.entry("cpu".into()).or_default();
            *samples = append_bounded(samples, load, CPU_HISTORY);
        }
        let mut order: Vec<String> = seen.into_iter().collect();
        order.sort();
        self.cpu_core_order = order;
    }

    pub fn update_firewall(&mut self, records: &[Resource]) {
        let mut history: HashMap<String, Vec<f64>> = self
            .firewall_rules
            .iter()
            .map(|rule| (rule.id.clone(), rule.history.clone()))
            .collect();
        let mut next_previous = HashMap::with_capacity(records.len());
        let mut rules = Vec::with_capacity(records.len());
        for record in records {
            if resource_bool(record, "disabled") {
                continue;
            }
            let id = row_id(record);
            let current = FirewallCounter {
                packets: resource_counter(record, &["packets"]),
                bytes: resource_counter(record, &["bytes"]),
            };
            next_previous.insert(id.clone(), current);
            let mut packet_delta = 0_u64;
            let mut byte_delta = 0_u64;
            if self.firewall_has_base
                && let Some(previous) = self.firewall_previous.get(&id)
            {
                if current.packets >= previous.packets {
                    packet_delta = current.packets - previous.packets;
                }
                if current.bytes >= previous.bytes {
                    byte_delta = current.bytes - previous.bytes;
                }
            }
            let activity = firewall_activity(packet_delta, byte_delta);
            let mut label = record.field("comment").unwrap_or("").trim().to_string();
            if label.is_empty() {
                label = format!(
                    "{} · {}",
                    record.field("chain").unwrap_or("").trim(),
                    record.field("action").unwrap_or("").trim()
                )
                .trim()
                .to_string();
            }
            if label.is_empty() {
                label.clone_from(&id);
            }
            let samples = history.remove(&id).unwrap_or_default();
            rules.push(FirewallRuleMetric {
                id,
                label,
                action: record.field("action").unwrap_or("").to_string(),
                packets: current.packets,
                bytes: current.bytes,
                recent_packets: packet_delta,
                recent_bytes: byte_delta,
                history: append_bounded(&samples, activity, FIREWALL_HISTORY),
            });
        }
        self.firewall_previous = next_previous;
        self.firewall_rules = rules;
        self.firewall_has_base = true;
    }

    /// Live rates from `/interface/monitor-traffic`.
    pub fn update_wan_monitor(&mut self, interface: &str, sample: &Resource) {
        if !interface.is_empty() {
            self.traffic_interface = interface.to_string();
        }
        let rx = resource_counter(sample, &["rx-bits-per-second", "rx-bits", "rx-byte"]);
        let tx = resource_counter(sample, &["tx-bits-per-second", "tx-bits", "tx-byte"]);
        #[allow(clippy::cast_precision_loss)]
        {
            self.traffic_rx_rate = rx as f64;
            self.traffic_tx_rate = tx as f64;
        }
        if self.traffic_samples.is_empty() {
            self.traffic_samples.push(TrafficSample::default());
        }
        self.traffic_samples.push(TrafficSample {
            rx: self.traffic_rx_rate,
            tx: self.traffic_tx_rate,
        });
        if self.traffic_samples.len() > TRAFFIC_HISTORY {
            let keep = self.traffic_samples.len() - TRAFFIC_HISTORY;
            self.traffic_samples.drain(..keep);
        }
        self.traffic_has_base = true;
        self.traffic_updated = Some(Instant::now());
    }

    pub fn update_wan(&mut self, iface: &Resource, at: Instant) {
        let rx = resource_counter(iface, &["rx-byte", "fp-rx-byte"]);
        let tx = resource_counter(iface, &["tx-byte", "fp-tx-byte"]);
        self.traffic_interface = iface.field("name").unwrap_or("").to_string();
        if self.traffic_has_base
            && let Some(previous) = self.traffic_updated
        {
            let elapsed = at.saturating_duration_since(previous).as_secs_f64();
            if elapsed > 0.0 {
                self.traffic_rx_rate = counter_rate(self.traffic_rx_bytes, rx, elapsed);
                self.traffic_tx_rate = counter_rate(self.traffic_tx_bytes, tx, elapsed);
                if self.traffic_samples.is_empty() {
                    self.traffic_samples.push(TrafficSample::default());
                }
                self.traffic_samples.push(TrafficSample {
                    rx: self.traffic_rx_rate,
                    tx: self.traffic_tx_rate,
                });
                if self.traffic_samples.len() > TRAFFIC_HISTORY {
                    let keep = self.traffic_samples.len() - TRAFFIC_HISTORY;
                    self.traffic_samples.drain(..keep);
                }
            }
        }
        self.traffic_rx_bytes = rx;
        self.traffic_tx_bytes = tx;
        self.traffic_updated = Some(at);
        self.traffic_has_base = true;
    }
}

/// WAN auto-detect: `PPPoE` > name contains wan > ether > bridge.
pub fn select_wan_interface(records: &[Resource]) -> Result<&Resource, &'static str> {
    let mut best_score = -1_i32;
    let mut best: Option<&Resource> = None;
    for record in records {
        if !resource_bool(record, "running") || resource_bool(record, "disabled") {
            continue;
        }
        let name = record.field("name").unwrap_or("").to_ascii_lowercase();
        let interface_type = record.field("type").unwrap_or("").to_ascii_lowercase();
        let score = if interface_type.contains("pppoe") || name.contains("pppoe") {
            100
        } else if name.contains("wan") {
            90
        } else if interface_type == "ether" {
            50
        } else if interface_type == "bridge" {
            20
        } else if interface_type == "loopback" {
            continue;
        } else {
            0
        };
        let traffic = resource_counter(record, &["rx-byte", "fp-rx-byte"])
            + resource_counter(record, &["tx-byte", "fp-tx-byte"]);
        let better = match best {
            None => true,
            Some(_) if score > best_score => true,
            Some(current) if score == best_score => {
                let current_traffic = resource_counter(current, &["rx-byte", "fp-rx-byte"])
                    + resource_counter(current, &["tx-byte", "fp-tx-byte"]);
                traffic > current_traffic
            }
            Some(_) => false,
        };
        if better {
            best_score = score;
            best = Some(record);
        }
    }
    if best_score < 0 {
        return Err("no active WAN interface detected");
    }
    best.ok_or("no active WAN interface detected")
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn counter_rate(previous: u64, current: u64, seconds: f64) -> f64 {
    if current < previous || seconds <= 0.0 {
        return 0.0;
    }
    (current - previous) as f64 * 8.0 / seconds
}

#[must_use]
pub fn resource_counter(resource: &Resource, names: &[&str]) -> u64 {
    for name in names {
        if let Some(value) = resource.field(name)
            && let Ok(parsed) = value.parse::<u64>()
        {
            return parsed;
        }
    }
    0
}

#[must_use]
pub fn resource_bool(resource: &Resource, name: &str) -> bool {
    parse_bool(resource.field(name).unwrap_or(""))
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "on" | "1"
    )
}

fn parse_f64(value: &str) -> Option<f64> {
    value.trim().trim_end_matches('%').parse().ok()
}

fn field(resource: &Resource, key: &str, fallback: &str) -> String {
    let value = resource.field(key).unwrap_or("");
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn row_id(resource: &Resource) -> String {
    if resource.id.is_empty() {
        resource
            .field("name")
            .or_else(|| resource.field("comment"))
            .unwrap_or("")
            .to_string()
    } else {
        resource.id.clone()
    }
}

fn append_bounded(samples: &[f64], value: f64, limit: usize) -> Vec<f64> {
    let mut next = samples.to_vec();
    next.push(value);
    if next.len() > limit {
        let keep = next.len() - limit;
        next.drain(..keep);
    }
    next
}

#[allow(clippy::cast_precision_loss)]
fn memory_percent(used: u64, total: u64) -> f64 {
    used as f64 * 100.0 / total as f64
}

#[allow(clippy::cast_precision_loss)]
fn firewall_activity(packet_delta: u64, byte_delta: u64) -> f64 {
    packet_delta as f64 + byte_delta as f64 / 1500.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource(id: &str, fields: &[(&str, &str)]) -> Resource {
        Resource {
            id: id.into(),
            fields: fields
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        }
    }

    #[test]
    fn wan_detection_prefers_running_pppoe() {
        let records = [
            resource(
                "*1",
                &[
                    ("name", "ether1"),
                    ("type", "ether"),
                    ("running", "true"),
                    ("rx-byte", "9000000"),
                    ("tx-byte", "1000000"),
                ],
            ),
            resource(
                "*2",
                &[
                    ("name", "pppoe-out2"),
                    ("type", "pppoe-out"),
                    ("running", "true"),
                    ("rx-byte", "100"),
                    ("tx-byte", "200"),
                ],
            ),
            resource(
                "*3",
                &[
                    ("name", "wan-backup"),
                    ("type", "ether"),
                    ("running", "false"),
                ],
            ),
        ];
        let selected = select_wan_interface(&records).expect("WAN");
        assert_eq!(selected.field("name"), Some("pppoe-out2"));
    }

    #[test]
    fn traffic_counter_rate_handles_reset() {
        assert!((counter_rate(1_000, 3_000, 2.0) - 8_000.0).abs() < f64::EPSILON);
        assert!((counter_rate(3_000, 1_000, 2.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn system_and_firewall_telemetry_history() {
        let mut dash = DashboardTelemetry::default();
        dash.update_system(&resource(
            "*s",
            &[("total-memory", "1000"), ("free-memory", "250")],
        ));
        dash.update_cpu(
            &[
                resource("*0", &[("cpu", "cpu0"), ("load", "25")]),
                resource("*1", &[("cpu", "cpu1"), ("load", "80")]),
            ],
            None,
        );
        assert_eq!(dash.memory_used_bytes, 750);
        assert_eq!(dash.memory_samples, vec![75.0]);
        assert_eq!(dash.cpu_core_order, vec!["cpu0", "cpu1"]);
        assert!((dash.cpu_core_loads["cpu1"] - 80.0).abs() < f64::EPSILON);

        dash.update_firewall(&[
            resource(
                "*1",
                &[
                    ("action", "accept"),
                    ("comment", "active"),
                    ("packets", "100"),
                    ("bytes", "10000"),
                ],
            ),
            resource(
                "*2",
                &[
                    ("action", "drop"),
                    ("comment", "dead"),
                    ("packets", "0"),
                    ("bytes", "0"),
                ],
            ),
        ]);
        dash.update_firewall(&[
            resource(
                "*1",
                &[
                    ("action", "accept"),
                    ("comment", "active"),
                    ("packets", "115"),
                    ("bytes", "25000"),
                ],
            ),
            resource(
                "*2",
                &[
                    ("action", "drop"),
                    ("comment", "dead"),
                    ("packets", "0"),
                    ("bytes", "0"),
                ],
            ),
        ]);
        assert_eq!(dash.firewall_rules.len(), 2);
        let active = dash
            .firewall_rules
            .iter()
            .find(|r| r.id == "*1")
            .expect("active");
        assert_eq!(active.recent_packets, 15);
        assert_eq!(active.recent_bytes, 15_000);
        assert_eq!(active.history.len(), 2);
        let dead = dash
            .firewall_rules
            .iter()
            .find(|r| r.id == "*2")
            .expect("dead");
        assert_eq!(dead.packets, 0);
        assert_eq!(dead.recent_packets, 0);
    }

    #[test]
    fn returning_resets_baseline_without_gap_spike() {
        let mut dash = DashboardTelemetry {
            traffic_interface: "pppoe-out2".into(),
            traffic_samples: vec![
                TrafficSample {
                    rx: 10_000_000.0,
                    tx: 1_000_000.0,
                },
                TrafficSample {
                    rx: 12_000_000.0,
                    tx: 2_000_000.0,
                },
            ],
            traffic_rx_bytes: 1_000,
            traffic_tx_bytes: 500,
            traffic_updated: Some(Instant::now()),
            traffic_has_base: true,
            ..DashboardTelemetry::default()
        };
        dash.activate();
        assert_eq!(dash.traffic_samples.len(), 2);
        assert!(!dash.traffic_has_base);

        let first = resource(
            "*1",
            &[
                ("name", "pppoe-out2"),
                ("rx-byte", "1000000"),
                ("tx-byte", "500000"),
            ],
        );
        let t0 = Instant::now();
        dash.update_wan(&first, t0);
        assert_eq!(dash.traffic_samples.len(), 2);
        assert!(dash.traffic_has_base);

        let second = resource(
            "*1",
            &[
                ("name", "pppoe-out2"),
                ("rx-byte", "1250000"),
                ("tx-byte", "625000"),
            ],
        );
        dash.update_wan(&second, t0 + std::time::Duration::from_secs(2));
        assert_eq!(dash.traffic_samples.len(), 3);
        assert!((dash.traffic_rx_rate - 1_000_000.0).abs() < 1.0);
        assert!((dash.traffic_tx_rate - 500_000.0).abs() < 1.0);
    }
}
