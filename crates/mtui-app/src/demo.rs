//! In-memory fixture profile so navigation can be learned without a router.

use std::collections::{BTreeMap, HashMap, HashSet};

use mtui_routeros::Resource;

use crate::app::AppCommand;
use crate::event::WorkerMsg;
use crate::write::MutationOp;

/// Saved-device list label and profile name for the fixture session.
pub const DEMO_PROFILE_NAME: &str = "Demo";

/// Connection target that never opens a TCP session.
pub const DEMO_URL: &str = "demo://router";

/// True when the login target should open the fixture profile.
#[must_use]
pub fn is_demo_target(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.eq_ignore_ascii_case("demo")
        || trimmed.to_ascii_lowercase().starts_with("demo:")
        || trimmed.eq_ignore_ascii_case(DEMO_URL)
}

/// Live fixture rows keyed by resource id.
#[derive(Debug, Clone)]
pub struct DemoStore {
    rows: HashMap<String, Vec<Resource>>,
    next_id: u32,
}

impl DemoStore {
    #[must_use]
    pub fn new() -> Self {
        let mut store = Self {
            rows: HashMap::new(),
            next_id: 40,
        };
        store.seed();
        store
    }

    #[must_use]
    pub fn system(&self) -> Resource {
        self.rows
            .get("system-resource")
            .and_then(|rows| rows.first())
            .cloned()
            .unwrap_or_default()
    }

    #[must_use]
    pub fn rows(&self, resource_id: &str) -> Vec<Resource> {
        match resource_id {
            "system-resource" | "identity" | "routerboard" | "health" | "package-update" => self
                .rows
                .get(resource_id)
                .cloned()
                .or_else(|| self.rows.get("system-resource").cloned())
                .unwrap_or_default(),
            other => self.rows.get(other).cloned().unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn lookup_options(
        &self,
        resource_id: &str,
        value_key: &str,
    ) -> Vec<mtui_core::LookupOption> {
        let mut options: Vec<mtui_core::LookupOption> = Vec::new();
        for row in self.rows(resource_id) {
            let Some(value) = row.field(value_key).filter(|value| !value.is_empty()) else {
                continue;
            };
            if options.iter().any(|option| option.value == value) {
                continue;
            }
            options.push(mtui_core::LookupOption::from_fields(value, &row.fields));
        }
        options.sort_by(|left, right| left.value.cmp(&right.value));
        options
    }

    #[cfg(test)]
    #[must_use]
    pub fn lookup_values(&self, resource_id: &str, value_key: &str) -> Vec<String> {
        self.lookup_options(resource_id, value_key)
            .into_iter()
            .map(|option| option.value)
            .collect()
    }

    pub fn apply(&mut self, op: &MutationOp) -> Result<(), String> {
        match op {
            MutationOp::Batch { ops } => {
                for inner in ops {
                    self.apply(inner)?;
                }
                Ok(())
            }
            MutationOp::Patch {
                endpoint,
                id,
                fields,
                ..
            } => {
                let Some(id) = id else {
                    self.patch_singleton(endpoint, fields);
                    return Ok(());
                };
                self.patch_id(endpoint, id, fields);
                Ok(())
            }
            MutationOp::Put { endpoint, fields } => {
                let id = self.alloc_id();
                let resource = resource_from_fields(&id, fields);
                if let Some(bucket) = self.bucket_for_endpoint(endpoint) {
                    bucket.push(resource);
                }
                Ok(())
            }
            MutationOp::Delete { endpoint, id } => {
                if let Some(bucket) = self.bucket_for_endpoint(endpoint) {
                    bucket.retain(|row| row.id != *id);
                }
                Ok(())
            }
            MutationOp::Command {
                endpoint,
                command,
                fields,
            } => {
                if endpoint.trim_end_matches('/').ends_with("/safe-mode") {
                    self.apply_safe_mode(command);
                    return Ok(());
                }
                let Some(id) = fields.get(".id") else {
                    return Ok(());
                };
                match command.as_str() {
                    "enable" => self.set_disabled(endpoint, id, false),
                    "disable" => self.set_disabled(endpoint, id, true),
                    "remove" | "undo" => {
                        if let Some(bucket) = self.bucket_for_endpoint(endpoint) {
                            bucket.retain(|row| row.id != *id);
                        }
                    }
                    "make-static" => self.set_field(endpoint, id, "dynamic", "false"),
                    "start" | "restart" => self.set_field(endpoint, id, "status", "running"),
                    "stop" | "kill" => self.set_field(endpoint, id, "status", "stopped"),
                    _ => {}
                }
                Ok(())
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn seed(&mut self) {
        self.rows.insert(
            "system-resource".into(),
            vec![resource(
                "",
                &[
                    ("identity", "demo-router"),
                    ("board-name", "CCR2004-16G-2S+"),
                    ("version", "7.18.2 (stable)"),
                    ("architecture-name", "arm64"),
                    ("cpu", "ARM"),
                    ("cpu-count", "4"),
                    ("cpu-load", "8"),
                    ("free-memory", "1879048192"),
                    ("total-memory", "2147483648"),
                    ("uptime", "1d2h"),
                    ("platform", "MikroTik"),
                ],
            )],
        );
        self.rows.insert(
            "routerboard".into(),
            vec![resource(
                "",
                &[
                    ("model", "CCR2004-16G-2S+"),
                    ("serial-number", "HEF123456789"),
                    ("current-firmware", "7.18.2"),
                    ("upgrade-firmware", "7.18.2"),
                    ("board-name", "CCR2004-16G-2S+"),
                ],
            )],
        );
        self.rows.insert(
            "routerboard-settings".into(),
            vec![resource(
                "",
                &[
                    ("boot-os", "router-os"),
                    ("boot-device", "nand-if-fail-then-ethernet"),
                    ("boot-protocol", "bootp"),
                    ("cpu-frequency", "auto"),
                    ("memory-frequency", "auto"),
                    ("enable-jumper-reset", "true"),
                    ("protected-routerboot", "disabled"),
                    ("silent-boot", "false"),
                    ("auto-upgrade", "false"),
                    ("force-backup-booter", "false"),
                ],
            )],
        );
        self.rows.insert(
            "routerboard-mode-button".into(),
            vec![resource(
                "",
                &[
                    ("enabled", "false"),
                    ("hold-time", "0.5s"),
                    ("on-event", ""),
                ],
            )],
        );
        self.rows.insert(
            "routerboard-reset-button".into(),
            vec![resource(
                "",
                &[("enabled", "false"), ("hold-time", "5s"), ("on-event", "")],
            )],
        );
        self.rows.insert(
            "watchdog".into(),
            vec![resource(
                "",
                &[
                    ("watchdog-timer", "true"),
                    ("watch-address", "192.0.2.1"),
                    ("watch-interval", "1m"),
                    ("no-ping-delay", "5m"),
                    ("ping-start-after", "5m"),
                    ("ping-timeout", "1s"),
                    ("automatic-supout", "true"),
                    ("auto-send-supout", "false"),
                ],
            )],
        );
        self.rows.insert(
            "ports".into(),
            vec![resource(
                "*port1",
                &[
                    ("name", "serial0"),
                    ("baud-rate", "115200"),
                    ("data-bits", "8"),
                    ("parity", "none"),
                    ("stop-bits", "1"),
                    ("flow-control", "none"),
                    ("used", "true"),
                ],
            )],
        );
        self.rows.insert(
            "system-console".into(),
            vec![resource(
                "*con1",
                &[
                    ("port", "serial0"),
                    ("term", "vt102"),
                    ("channel", "0"),
                    ("disabled", "false"),
                    ("used", "true"),
                ],
            )],
        );
        self.rows.insert(
            "leds".into(),
            vec![resource(
                "*led1",
                &[
                    ("type", "interface-activity"),
                    ("interface", "ether1"),
                    ("leds", "user-led"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "led-settings".into(),
            vec![resource("", &[("all-leds-off", "never")])],
        );
        self.rows.insert(
            "special-login".into(),
            vec![resource(
                "*sl1",
                &[
                    ("user", "serial"),
                    ("port", "serial0"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "users".into(),
            vec![resource(
                "*u1",
                &[
                    ("name", "admin"),
                    ("group", "full"),
                    ("address", "0.0.0.0/0"),
                    ("inactivity-policy", "none"),
                    ("inactivity-timeout", "10m"),
                    ("disabled", "false"),
                    ("last-logged-in", "aug/01/2026 12:00:00"),
                ],
            )],
        );
        self.rows.insert(
            "packages".into(),
            vec![
                resource(
                    "*pkg1",
                    &[
                        ("name", "routeros"),
                        ("version", "7.18.2"),
                        ("build-time", "2025-01-15"),
                        ("disabled", "false"),
                    ],
                ),
                resource(
                    "*pkg2",
                    &[
                        ("name", "container"),
                        ("version", "7.18.2"),
                        ("build-time", "2025-01-15"),
                        ("disabled", "false"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "interfaces".into(),
            vec![
                resource(
                    "*1",
                    &[
                        ("name", "ether1"),
                        ("type", "ether"),
                        ("mtu", "1500"),
                        ("mac-address", "74:4D:28:00:00:01"),
                        ("running", "true"),
                        ("disabled", "false"),
                        ("comment", "WAN"),
                    ],
                ),
                resource(
                    "*2",
                    &[
                        ("name", "ether2"),
                        ("type", "ether"),
                        ("mtu", "1500"),
                        ("mac-address", "74:4D:28:00:00:02"),
                        ("running", "true"),
                        ("disabled", "false"),
                        ("comment", "LAN"),
                    ],
                ),
                resource(
                    "*3",
                    &[
                        ("name", "bridge"),
                        ("type", "bridge"),
                        ("mtu", "1500"),
                        ("running", "true"),
                        ("disabled", "false"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "lte".into(),
            vec![resource(
                "*lte1",
                &[
                    ("name", "lte1"),
                    ("default-name", "lte1"),
                    ("type", "lte"),
                    ("mtu", "1500"),
                    ("network-mode", "3g,lte"),
                    ("apn-profiles", "default"),
                    ("running", "true"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "lte-apn".into(),
            vec![
                resource(
                    "*apn1",
                    &[
                        ("name", "default"),
                        ("apn", "internet"),
                        ("authentication", "none"),
                        ("ip-type", "ipv4"),
                        ("use-network-apn", "true"),
                        ("use-peer-dns", "true"),
                        ("add-default-route", "true"),
                        ("default-route-distance", "2"),
                    ],
                ),
                resource(
                    "*apn2",
                    &[
                        ("name", "carrier"),
                        ("apn", "lte.provider"),
                        ("authentication", "chap"),
                        ("user", "user"),
                        ("password", "secret-apn"),
                        ("ip-type", "ipv4"),
                        ("use-network-apn", "false"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "addresses".into(),
            vec![
                resource(
                    "*10",
                    &[
                        ("address", "192.0.2.1/24"),
                        ("network", "192.0.2.0"),
                        ("interface", "ether1"),
                        ("disabled", "false"),
                    ],
                ),
                resource(
                    "*11",
                    &[
                        ("address", "10.0.0.1/24"),
                        ("network", "10.0.0.0"),
                        ("interface", "bridge"),
                        ("disabled", "false"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "dhcp-servers".into(),
            vec![resource(
                "*20",
                &[
                    ("name", "dhcp1"),
                    ("interface", "bridge"),
                    ("lease-time", "30m"),
                    ("address-pool", "lan-pool"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "dhcp-networks".into(),
            vec![resource(
                "*21",
                &[
                    ("address", "10.0.0.0/24"),
                    ("gateway", "10.0.0.1"),
                    ("dns-server", "10.0.0.1"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "dhcp-leases".into(),
            vec![
                resource(
                    "*22",
                    &[
                        ("address", "10.0.0.20"),
                        ("mac-address", "AA:BB:CC:00:00:20"),
                        ("server", "dhcp1"),
                        ("status", "bound"),
                        ("host-name", "laptop"),
                        ("dynamic", "true"),
                    ],
                ),
                resource(
                    "*23",
                    &[
                        ("address", "10.0.0.21"),
                        ("mac-address", "AA:BB:CC:00:00:21"),
                        ("server", "dhcp1"),
                        ("status", "bound"),
                        ("host-name", "phone"),
                        ("dynamic", "true"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "firewall-filter".into(),
            vec![
                resource(
                    "*30",
                    &[
                        ("chain", "input"),
                        ("action", "accept"),
                        ("comment", "established"),
                        ("connection-state", "established,related"),
                        ("disabled", "false"),
                    ],
                ),
                resource(
                    "*31",
                    &[
                        ("chain", "forward"),
                        ("action", "drop"),
                        ("comment", "drop invalid"),
                        ("connection-state", "invalid"),
                        ("disabled", "false"),
                    ],
                ),
                resource(
                    "*32",
                    &[
                        ("chain", "input"),
                        ("action", "drop"),
                        ("comment", "drop wan input"),
                        ("in-interface", "ether1"),
                        ("disabled", "true"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "firewall-nat".into(),
            vec![resource(
                "*33",
                &[
                    ("chain", "srcnat"),
                    ("action", "masquerade"),
                    ("out-interface", "ether1"),
                    ("comment", "WAN masquerade"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "ipv6-firewall-connections".into(),
            vec![
                resource(
                    "*36",
                    &[
                        ("src-address", "2001:db8:1::10"),
                        ("dst-address", "2001:db8:2::1"),
                        ("protocol", "tcp"),
                        ("src-port", "53100"),
                        ("dst-port", "443"),
                        ("tcp-state", "established"),
                        ("timeout", "23h59m"),
                        ("orig-rate", "1200"),
                        ("repl-rate", "8500"),
                        ("connection-mark", ""),
                    ],
                ),
                resource(
                    "*37",
                    &[
                        ("src-address", "2001:db8:1::20"),
                        ("dst-address", "2001:db8::53"),
                        ("protocol", "udp"),
                        ("src-port", "53222"),
                        ("dst-port", "53"),
                        ("tcp-state", ""),
                        ("timeout", "10s"),
                        ("orig-rate", "40"),
                        ("repl-rate", "80"),
                        ("connection-mark", ""),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "queue-simple".into(),
            vec![resource(
                "*34",
                &[
                    ("name", "guest"),
                    ("target", "10.0.0.0/24"),
                    ("max-limit", "20M/20M"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "queue-tree".into(),
            vec![resource(
                "*35",
                &[
                    ("name", "wan-out"),
                    ("parent", "global"),
                    ("max-limit", "100M"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "routes".into(),
            vec![resource(
                "*36",
                &[
                    ("dst-address", "0.0.0.0/0"),
                    ("gateway", "192.0.2.254"),
                    ("distance", "1"),
                    ("active", "true"),
                ],
            )],
        );
        self.rows.insert(
            "ospf-interface-templates".into(),
            vec![resource(
                "*oi0",
                &[
                    ("instance", "default"),
                    ("area", "backbone"),
                    ("interfaces", "ether2"),
                    ("type", "broadcast"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "ospf-interfaces".into(),
            vec![
                resource(
                    "*oi1",
                    &[
                        ("address", "10.0.0.1%ether2"),
                        ("area", "backbone"),
                        ("state", "dr"),
                        ("network-type", "broadcast"),
                        ("cost", "10"),
                        ("priority", "128"),
                        ("dr", "10.0.0.1"),
                        ("bdr", "10.0.0.2"),
                        ("hello-interval", "10s"),
                        ("dead-interval", "40s"),
                        ("dynamic", "true"),
                    ],
                ),
                resource(
                    "*oi2",
                    &[
                        ("address", "10.0.0.1%lo"),
                        ("area", "backbone"),
                        ("state", "passive"),
                        ("network-type", "broadcast"),
                        ("cost", "1"),
                        ("dynamic", "true"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "logs".into(),
            vec![
                resource(
                    "*l1",
                    &[
                        ("time", "12:01:00"),
                        ("topics", "system,info"),
                        ("message", "demo profile ready"),
                    ],
                ),
                resource(
                    "*l2",
                    &[
                        ("time", "12:02:00"),
                        ("topics", "dhcp,info"),
                        ("message", "assigned 10.0.0.20 to laptop"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "logging-actions".into(),
            vec![
                resource("*la1", &[("name", "memory"), ("target", "memory")]),
                resource("*la2", &[("name", "disk"), ("target", "disk")]),
                resource("*la3", &[("name", "echo"), ("target", "echo")]),
                resource("*la4", &[("name", "email"), ("target", "email")]),
                resource(
                    "*la5",
                    &[
                        ("name", "remote"),
                        ("target", "remote"),
                        ("remote", "192.0.2.10"),
                        ("remote-port", "514"),
                        ("remote-protocol", "udp"),
                        ("remote-log-format", "syslog"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "logging".into(),
            vec![
                resource(
                    "*lr1",
                    &[
                        ("topics", "info"),
                        ("action", "memory"),
                        ("prefix", ""),
                        ("disabled", "false"),
                    ],
                ),
                resource(
                    "*lr2",
                    &[
                        ("topics", "error"),
                        ("action", "remote"),
                        ("prefix", ""),
                        ("disabled", "false"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "ntp-server".into(),
            vec![resource(
                "",
                &[
                    ("enabled", "false"),
                    ("broadcast", "false"),
                    ("multicast", "false"),
                    ("manycast", "false"),
                    ("vrf", "main"),
                    ("use-local-clock", "false"),
                    ("local-clock-stratum", "5"),
                    ("broadcast-addresses", ""),
                    ("auth-key", "none"),
                ],
            )],
        );
        self.rows.insert(
            "ntp-keys".into(),
            vec![resource("*nk1", &[("key-id", "1")])],
        );
        self.rows.insert(
            "traffic-flow".into(),
            vec![resource(
                "",
                &[
                    ("enabled", "false"),
                    ("interfaces", "all"),
                    ("cache-entries", "4k"),
                    ("active-flow-timeout", "30m"),
                    ("inactive-flow-timeout", "15s"),
                    ("packet-sampling", "false"),
                    ("sampling-interval", "0"),
                    ("sampling-space", "0"),
                ],
            )],
        );
        self.rows.insert(
            "traffic-flow-targets".into(),
            vec![resource(
                "*tf1",
                &[
                    ("src-address", "0.0.0.0"),
                    ("dst-address", "192.0.2.10"),
                    ("port", "2055"),
                    ("version", "ipfix"),
                    ("v9-template-refresh", "20"),
                    ("v9-template-timeout", "1m"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "traffic-flow-ipfix".into(),
            vec![resource(
                "",
                &[
                    ("bytes", "true"),
                    ("src-address", "true"),
                    ("dst-address", "true"),
                    ("protocol", "true"),
                    ("nat-events", "false"),
                    ("src-port", "true"),
                    ("dst-port", "true"),
                ],
            )],
        );
        self.rows.insert(
            "igmp-proxy".into(),
            vec![resource(
                "",
                &[
                    ("query-interval", "2m5s"),
                    ("query-response-interval", "10s"),
                    ("last-member-query-interval", "1s"),
                    ("robustness", "2"),
                    ("quick-leave", "false"),
                ],
            )],
        );
        self.rows.insert(
            "igmp-proxy-interfaces".into(),
            vec![
                resource(
                    "*ig1",
                    &[
                        ("interface", "ether1"),
                        ("upstream", "true"),
                        ("threshold", "1"),
                        ("alternative-subnets", "192.168.50.0/24"),
                        ("disabled", "false"),
                        ("querier", "false"),
                        ("source-ip-address", "192.0.2.1"),
                    ],
                ),
                resource(
                    "*ig2",
                    &[
                        ("interface", "ether2"),
                        ("upstream", "false"),
                        ("threshold", "1"),
                        ("alternative-subnets", ""),
                        ("disabled", "false"),
                        ("querier", "true"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "igmp-proxy-mfc".into(),
            vec![resource(
                "*mfc1",
                &[
                    ("group", "239.1.1.1"),
                    ("source", "192.0.2.50"),
                    ("upstream-interface", "ether1"),
                    ("downstream-interfaces", "ether2"),
                    ("packets", "12"),
                    ("bytes", "1440"),
                    ("wrong-packets", "0"),
                ],
            )],
        );
        self.rows.insert(
            "romon".into(),
            vec![resource(
                "",
                &[
                    ("enabled", "false"),
                    ("id", "00:00:00:00:00:00"),
                    ("secrets", "demo-romon-secret"),
                    ("current-id", "74:4D:28:00:00:01"),
                ],
            )],
        );
        self.rows.insert(
            "romon-ports".into(),
            vec![
                resource(
                    "*rp1",
                    &[
                        ("interface", "all"),
                        ("forbid", "false"),
                        ("cost", "100"),
                        ("secrets", ""),
                        ("disabled", "false"),
                    ],
                ),
                resource(
                    "*rp2",
                    &[
                        ("interface", "ether1"),
                        ("forbid", "false"),
                        ("cost", "200"),
                        ("secrets", ""),
                        ("disabled", "false"),
                        ("comment", "WAN"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "graphing".into(),
            vec![resource(
                "",
                &[("store-every", "5min"), ("page-refresh", "300")],
            )],
        );
        self.rows.insert(
            "graphing-interface".into(),
            vec![resource(
                "*gi1",
                &[
                    ("interface", "all"),
                    ("allow-address", "0.0.0.0/0"),
                    ("store-on-disk", "true"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "graphing-queue".into(),
            vec![resource(
                "*gq1",
                &[
                    ("simple-queue", "all"),
                    ("allow-address", "0.0.0.0/0"),
                    ("allow-target", "true"),
                    ("store-on-disk", "true"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "graphing-resource".into(),
            vec![resource(
                "*gr1",
                &[
                    ("allow-address", "192.0.2.0/24"),
                    ("store-on-disk", "false"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "smb".into(),
            vec![resource(
                "",
                &[
                    ("enabled", "auto"),
                    ("domain", "MSHOME"),
                    ("comment", "MikrotikSMB"),
                    ("allow-guests", "false"),
                ],
            )],
        );
        self.rows.insert(
            "smb-users".into(),
            vec![
                resource(
                    "*smb1",
                    &[
                        ("name", "guest"),
                        ("password", ""),
                        ("read-only", "true"),
                        ("disabled", "true"),
                        ("default", "true"),
                        ("comment", ""),
                    ],
                ),
                resource(
                    "*smb2",
                    &[
                        ("name", "mtuser"),
                        ("password", "demo-secret"),
                        ("read-only", "false"),
                        ("disabled", "false"),
                        ("default", "false"),
                        ("comment", "office"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "smb-shares".into(),
            vec![
                resource(
                    "*smbs1",
                    &[
                        ("name", "pub"),
                        ("directory", "/pub"),
                        ("require-encryption", "false"),
                        ("read-only", "false"),
                        ("valid-users", ""),
                        ("invalid-users", ""),
                        ("disabled", "true"),
                        ("default", "true"),
                        ("comment", "default share"),
                    ],
                ),
                resource(
                    "*smbs2",
                    &[
                        ("name", "backup"),
                        ("directory", "backup"),
                        ("require-encryption", "false"),
                        ("read-only", "false"),
                        ("valid-users", "mtuser"),
                        ("invalid-users", ""),
                        ("disabled", "false"),
                        ("default", "false"),
                        ("comment", ""),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "files".into(),
            vec![
                resource("*f1", &[("name", "/pub"), ("type", "directory")]),
                resource("*f2", &[("name", "backup"), ("type", "directory")]),
            ],
        );
        self.rows.insert(
            "license".into(),
            vec![resource(
                "",
                &[
                    ("software-id", "ABCD-EFGH"),
                    ("nlevel", "6"),
                    ("features", ""),
                    ("system-id", ""),
                    ("level", ""),
                ],
            )],
        );
        self.rows.insert(
            "disks".into(),
            vec![
                resource(
                    "*d1",
                    &[
                        ("slot", "usb1"),
                        ("type", "hardware"),
                        ("model", "USB DISK"),
                        ("serial", "DEMO123"),
                        ("size", "32000000000"),
                        ("free", "30000000000"),
                        ("fs", "ext4"),
                        ("state", "ok"),
                        ("disabled", "false"),
                    ],
                ),
                resource(
                    "*d2",
                    &[
                        ("slot", "raid1"),
                        ("type", "raid"),
                        ("raid-type", "1"),
                        ("raid-device-count", "2"),
                        ("model", "RAID1"),
                        ("fs", "ext4"),
                        ("disabled", "false"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "device-mode".into(),
            vec![resource(
                "",
                &[
                    ("mode", "advanced"),
                    ("flagged", "false"),
                    ("flagging-enabled", "true"),
                    ("container", "false"),
                    ("scheduler", "true"),
                    ("traffic-gen", "false"),
                    ("fetch", "true"),
                    ("allowed-versions", "7.13+,6.49.8+"),
                    ("attempt-count", "0"),
                ],
            )],
        );
        self.rows.insert(
            "veth".into(),
            vec![resource(
                "*v1",
                &[
                    ("name", "veth1"),
                    ("address", "172.17.0.2/24"),
                    ("gateway", "172.17.0.1"),
                    ("dhcp", "false"),
                    ("running", "true"),
                    ("disabled", "false"),
                ],
            )],
        );
        self.rows.insert(
            "container-config".into(),
            vec![resource(
                "",
                &[
                    ("registry-url", "https://registry-1.docker.io"),
                    ("tmpdir", "disk1/tmp"),
                    ("username", ""),
                    ("password", "registry-demo"),
                    ("memory-current", "0"),
                ],
            )],
        );
        self.rows.insert(
            "container-envs".into(),
            vec![
                resource(
                    "*e1",
                    &[
                        ("list", "ENV_PIHOLE"),
                        ("key", "TZ"),
                        ("value", "Europe/Riga"),
                    ],
                ),
                resource(
                    "*e2",
                    &[
                        ("list", "ENV_PIHOLE"),
                        ("key", "FTLCONF_webserver_api_password"),
                        ("value", "demo"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "container-mounts".into(),
            vec![resource(
                "*m1",
                &[
                    ("list", "MOUNT_PIHOLE"),
                    ("src", "disk1/volumes/pihole"),
                    ("dst", "/etc/pihole"),
                ],
            )],
        );
        self.rows.insert(
            "containers".into(),
            vec![
                resource(
                    "*c1",
                    &[
                        ("name", "pihole"),
                        ("tag", "pihole/pihole:latest"),
                        ("interface", "veth1"),
                        ("status", "stopped"),
                        ("arch", "arm64"),
                        ("os", "linux"),
                        ("root-dir", "disk1/images/pihole"),
                        ("start-on-boot", "true"),
                        ("logging", "true"),
                    ],
                ),
                resource(
                    "*c2",
                    &[
                        ("name", "alpine"),
                        ("tag", "alpine:latest"),
                        ("interface", "veth1"),
                        ("status", "running"),
                        ("arch", "arm64"),
                        ("os", "linux"),
                        ("root-dir", "disk1/images/alpine"),
                        ("start-on-boot", "false"),
                        ("logging", "false"),
                    ],
                ),
            ],
        );
        self.rows.insert(
            "apps".into(),
            vec![resource(
                "*a1",
                &[
                    ("name", "adguard"),
                    ("status", "stopped"),
                    ("running", "false"),
                    ("network", "internal"),
                    ("ui-url", "http://172.17.0.2"),
                ],
            )],
        );
        self.rows.insert(
            "safe-mode".into(),
            vec![resource(
                "",
                &[
                    ("enabled", "false"),
                    ("current", "false"),
                    ("owner", ""),
                    ("user", ""),
                ],
            )],
        );
        self.rows.insert(
            "history".into(),
            vec![
                resource(
                    "*h1",
                    &[
                        ("time", "aug/25/2026 01:00:00"),
                        ("action", "set"),
                        ("by", "admin"),
                        ("policy", "write"),
                        ("floating-undo", "false"),
                    ],
                ),
                resource(
                    "*h2",
                    &[
                        ("time", "aug/25/2026 01:02:00"),
                        ("action", "set"),
                        ("by", "admin"),
                        ("policy", "write"),
                        ("floating-undo", "true"),
                        ("flags", "F"),
                    ],
                ),
            ],
        );
    }

    fn apply_safe_mode(&mut self, command: &str) {
        let row = self
            .rows
            .entry("safe-mode".into())
            .or_insert_with(|| vec![resource("", &[])]);
        let Some(row) = row.first_mut() else {
            return;
        };
        match command {
            "take" => {
                row.fields.insert("enabled".into(), "true".into());
                row.fields.insert("current".into(), "true".into());
                row.fields.insert("owner".into(), "api".into());
                row.fields.insert("user".into(), "demo".into());
            }
            "release" | "unroll" => {
                row.fields.insert("enabled".into(), "false".into());
                row.fields.insert("current".into(), "false".into());
                row.fields.insert("owner".into(), String::new());
                row.fields.insert("user".into(), String::new());
            }
            _ => {}
        }
    }

    fn alloc_id(&mut self) -> String {
        let id = format!("*{}", self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    fn bucket_for_endpoint(&mut self, endpoint: &str) -> Option<&mut Vec<Resource>> {
        let id = resource_id_for_endpoint(endpoint)?;
        self.rows.get_mut(id)
    }

    fn patch_id(&mut self, endpoint: &str, id: &str, fields: &BTreeMap<String, String>) {
        if let Some(row) = self
            .bucket_for_endpoint(endpoint)
            .and_then(|bucket| bucket.iter_mut().find(|row| row.id == id))
        {
            for (key, value) in fields {
                row.fields.insert(key.clone(), value.clone());
            }
        }
    }

    fn patch_singleton(&mut self, endpoint: &str, fields: &BTreeMap<String, String>) {
        if let Some(row) = self
            .bucket_for_endpoint(endpoint)
            .and_then(|bucket| bucket.first_mut())
        {
            for (key, value) in fields {
                row.fields.insert(key.clone(), value.clone());
            }
        }
    }

    fn set_disabled(&mut self, endpoint: &str, id: &str, disabled: bool) {
        self.set_field(
            endpoint,
            id,
            "disabled",
            if disabled { "true" } else { "false" },
        );
    }

    fn set_field(&mut self, endpoint: &str, id: &str, key: &str, value: &str) {
        if let Some(row) = self
            .bucket_for_endpoint(endpoint)
            .and_then(|bucket| bucket.iter_mut().find(|row| row.id == id))
        {
            row.fields.insert(key.to_string(), value.to_string());
        }
    }
}

impl Default for DemoStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Serve a command from the fixture store. `None` means the runtime should
/// handle the command itself (clipboard, quit, file I/O).
#[allow(clippy::too_many_lines)]
pub fn handle(store: &mut DemoStore, cmd: &AppCommand) -> Option<Vec<WorkerMsg>> {
    match cmd {
        AppCommand::FetchResource {
            session,
            request_id,
            generation,
            resource_id,
        } => Some(vec![WorkerMsg::ResourceResult {
            session: *session,
            request_id: *request_id,
            generation: *generation,
            resource_id: resource_id.clone(),
            rows: store.rows(resource_id),
            error: None,
        }]),
        AppCommand::FetchDashboard {
            session,
            request_id,
            generation,
        } => Some(vec![WorkerMsg::DashboardResult {
            session: *session,
            request_id: *request_id,
            generation: *generation,
            cpu: Vec::new(),
            cpu_error: None,
            system: store.rows("system-resource").into_iter().next(),
            system_error: None,
            interfaces: store.rows("interfaces"),
            interface_error: None,
            firewall: store.rows("firewall-filter"),
            firewall_error: None,
        }]),
        AppCommand::FetchAccess {
            session,
            generation,
            ..
        } => Some(vec![WorkerMsg::AccessResult {
            session: *session,
            generation: *generation,
            users: vec![resource("*1", &[("name", "demo"), ("group", "full")])],
            groups: vec![resource(
                "*g",
                &[
                    ("name", "full"),
                    ("policy", "read,write,policy,test,reboot,sniff,api"),
                ],
            )],
            error: None,
        }]),
        AppCommand::ProbeMenuPaths {
            session,
            generation,
        } => Some(vec![WorkerMsg::MenuPathsResult {
            session: *session,
            generation: *generation,
            missing_ids: HashSet::new(),
            error: None,
        }]),
        AppCommand::FetchHeader {
            session,
            request_id,
            generation,
        } => Some(vec![WorkerMsg::HeaderResult {
            session: *session,
            request_id: *request_id,
            generation: *generation,
            system: store.rows("system-resource").into_iter().next(),
            system_error: None,
            interfaces: store.rows("interfaces"),
            interface_error: None,
        }]),
        AppCommand::FetchLookup {
            session,
            request_id,
            generation,
            resource_id,
            value_key,
        } => Some(vec![WorkerMsg::LookupResult {
            session: *session,
            request_id: *request_id,
            generation: *generation,
            options: store.lookup_options(resource_id, value_key),
            error: None,
        }]),
        AppCommand::FetchFormRecord {
            session,
            request_id,
            generation,
            resource_id,
            id,
            ..
        } => {
            let row = store
                .rows(resource_id)
                .into_iter()
                .find(|row| row.id == *id);
            Some(vec![WorkerMsg::FormRecordResult {
                session: *session,
                request_id: *request_id,
                generation: *generation,
                resource_id: resource_id.clone(),
                id: id.clone(),
                fields: row.as_ref().map(|row| row.fields.clone()),
                error: row.is_none().then(|| "no such item".into()),
            }])
        }
        AppCommand::Mutate {
            session,
            request_id,
            generation,
            op,
        } => {
            let error = store.apply(op).err();
            Some(vec![WorkerMsg::MutateResult {
                session: *session,
                request_id: *request_id,
                generation: *generation,
                error,
            }])
        }
        AppCommand::FetchSafeMode {
            session,
            generation,
        } => Some(vec![WorkerMsg::SafeModeResult {
            session: *session,
            generation: *generation,
            row: store.rows("safe-mode").into_iter().next(),
            error: None,
        }]),
        AppCommand::FetchTorch {
            session,
            generation,
            ..
        } => Some(vec![WorkerMsg::TorchResult {
            session: *session,
            generation: *generation,
            rows: Vec::new(),
            error: Some("Demo profile has no live probes".into()),
            done: true,
        }]),
        AppCommand::FetchPing {
            session,
            generation,
            ..
        }
        | AppCommand::FetchTraceroute {
            session,
            generation,
            ..
        }
        | AppCommand::FetchProbe {
            session,
            generation,
            ..
        } => Some(vec![WorkerMsg::PingTraceResult {
            session: *session,
            generation: *generation,
            rows: Vec::new(),
            error: Some("Demo profile has no live probes".into()),
            done: true,
        }]),
        _ => None,
    }
}

fn resource(id: &str, fields: &[(&str, &str)]) -> Resource {
    Resource {
        id: id.to_string(),
        fields: fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
    }
}

fn resource_from_fields(id: &str, fields: &BTreeMap<String, String>) -> Resource {
    Resource {
        id: id.to_string(),
        fields: fields
            .iter()
            .filter(|(key, _)| key.as_str() != ".id")
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    }
}

fn resource_id_for_endpoint(endpoint: &str) -> Option<&'static str> {
    mtui_core::ALL_RESOURCES
        .iter()
        .find(|spec| spec.endpoint() == endpoint)
        .map(|spec| spec.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_target_accepts_scheme_and_bare_name() {
        assert!(is_demo_target("demo"));
        assert!(is_demo_target("demo://router"));
        assert!(is_demo_target("DEMO://x"));
        assert!(!is_demo_target("192.168.88.1"));
    }

    #[test]
    fn demo_store_has_operator_lists() {
        let store = DemoStore::new();
        assert_eq!(store.rows("interfaces").len(), 3);
        assert_eq!(store.rows("firewall-filter").len(), 3);
        assert_eq!(store.rows("dhcp-leases").len(), 2);
        assert_eq!(store.rows("packages")[0].field("name"), Some("routeros"));
        assert!(
            store
                .rows("packages")
                .iter()
                .any(|row| row.field("name") == Some("container"))
        );
        assert_eq!(
            store.rows("system-resource")[0].field("architecture-name"),
            Some("arm64")
        );
        assert_eq!(store.rows("veth").len(), 1);
        assert_eq!(store.rows("containers").len(), 2);
        assert_eq!(
            store.rows("system-resource")[0].field("version"),
            Some("7.18.2 (stable)")
        );
        assert_eq!(store.rows("history").len(), 2);
        assert_eq!(store.rows("ipv6-firewall-connections").len(), 2);
        assert_eq!(
            store.rows("ipv6-firewall-connections")[0].field("src-address"),
            Some("2001:db8:1::10")
        );
        assert_eq!(store.rows("ospf-interfaces").len(), 2);
        assert_eq!(store.rows("ospf-interfaces")[0].field("state"), Some("dr"));
        assert_eq!(
            store.rows("ospf-interface-templates")[0].field("interfaces"),
            Some("ether2")
        );
        assert_eq!(store.rows("smb-users").len(), 2);
        assert_eq!(store.rows("smb-shares").len(), 2);
        assert_eq!(store.rows("smb-users")[1].field("name"), Some("mtuser"));
        assert_eq!(store.rows("lte").len(), 1);
        assert_eq!(store.rows("lte-apn").len(), 2);
        assert_eq!(
            store.lookup_values("lte-apn", "name"),
            ["carrier", "default"]
        );
    }

    #[test]
    fn demo_remove_drops_ipv6_firewall_connection() {
        let mut store = DemoStore::new();
        store
            .apply(&MutationOp::Delete {
                endpoint: "/ipv6/firewall/connection".into(),
                id: "*36".into(),
            })
            .expect("delete");
        let rows = store.rows("ipv6-firewall-connections");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "*37");
    }

    #[test]
    fn demo_container_start_sets_running() {
        let mut store = DemoStore::new();
        store
            .apply(&MutationOp::Command {
                endpoint: "/container".into(),
                command: "start".into(),
                fields: BTreeMap::from([(".id".into(), "*c1".into())]),
            })
            .expect("start");
        let row = store
            .rows("containers")
            .into_iter()
            .find(|row| row.id == "*c1")
            .expect("pihole");
        assert_eq!(row.field("status"), Some("running"));
    }

    #[test]
    fn demo_logging_actions_include_remote_syslog() {
        let store = DemoStore::new();
        assert!(
            store.rows("logging-actions").iter().any(|row| {
                row.field("target") == Some("remote") && row.field("remote") == Some("192.0.2.10")
            }),
            "demo should seed a remote syslog action"
        );
        assert!(!store.rows("logging").is_empty());
    }

    #[test]
    fn demo_ntp_server_is_disabled_singleton() {
        let store = DemoStore::new();
        let rows = store.rows("ntp-server");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "");
        assert_eq!(rows[0].field("enabled"), Some("false"));
        assert_eq!(rows[0].field("auth-key"), Some("none"));
        assert_eq!(
            store.lookup_values("ntp-keys", "key-id"),
            vec!["1".to_string()]
        );
    }

    #[test]
    fn demo_traffic_flow_and_igmp_proxy_are_seeded() {
        let store = DemoStore::new();
        let flow = store.rows("traffic-flow");
        assert_eq!(flow.len(), 1);
        assert_eq!(flow[0].id, "");
        assert_eq!(flow[0].field("enabled"), Some("false"));
        assert_eq!(flow[0].field("interfaces"), Some("all"));
        assert_eq!(flow[0].field("cache-entries"), Some("4k"));

        let targets = store.rows("traffic-flow-targets");
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].field("dst-address"), Some("192.0.2.10"));
        assert_eq!(targets[0].field("version"), Some("ipfix"));
        assert_eq!(targets[0].field("port"), Some("2055"));

        assert_eq!(
            store.rows("traffic-flow-ipfix")[0].field("bytes"),
            Some("true")
        );
        assert_eq!(
            store.rows("igmp-proxy")[0].field("query-interval"),
            Some("2m5s")
        );
        assert_eq!(store.rows("igmp-proxy-interfaces").len(), 2);
        assert!(
            store
                .rows("igmp-proxy-interfaces")
                .iter()
                .any(|row| row.field("upstream") == Some("true"))
        );
        assert_eq!(
            store.rows("igmp-proxy-mfc")[0].field("group"),
            Some("239.1.1.1")
        );
    }

    #[test]
    fn demo_romon_and_graphing_are_seeded() {
        let mut store = DemoStore::new();
        let romon = store.rows("romon");
        assert_eq!(romon.len(), 1);
        assert_eq!(romon[0].id, "");
        assert_eq!(romon[0].field("enabled"), Some("false"));
        assert_eq!(romon[0].field("secrets"), Some("demo-romon-secret"));
        assert_eq!(store.rows("romon-ports").len(), 2);
        assert_eq!(
            store.lookup_values("romon-ports", "interface"),
            vec!["all".to_string(), "ether1".to_string()]
        );
        assert_eq!(store.rows("graphing")[0].field("store-every"), Some("5min"));
        assert_eq!(store.rows("graphing-interface").len(), 1);
        assert_eq!(
            store.rows("graphing-queue")[0].field("simple-queue"),
            Some("all")
        );
        assert_eq!(
            store.rows("graphing-resource")[0].field("allow-address"),
            Some("192.0.2.0/24")
        );
        assert_eq!(
            store.apply(&MutationOp::Patch {
                endpoint: "/tool/romon".into(),
                id: None,
                fields: BTreeMap::from([("enabled".into(), "true".into())]),
            }),
            Ok(())
        );
        assert_eq!(store.rows("romon")[0].field("enabled"), Some("true"));
    }

    #[test]
    fn demo_history_undo_removes_that_row() {
        let mut store = DemoStore::new();
        store
            .apply(&MutationOp::Command {
                endpoint: "/system/history".into(),
                command: "undo".into(),
                fields: BTreeMap::from([(".id".into(), "*h1".into())]),
            })
            .expect("undo");
        let ids: Vec<_> = store
            .rows("history")
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(ids, vec!["*h2".to_string()]);
    }

    #[test]
    fn demo_safe_mode_take_and_print() {
        let mut store = DemoStore::new();
        store
            .apply(&MutationOp::Command {
                endpoint: "/safe-mode".into(),
                command: "take".into(),
                fields: BTreeMap::new(),
            })
            .expect("take");
        let row = store.rows("safe-mode").into_iter().next().expect("row");
        assert_eq!(row.field("enabled"), Some("true"));
        assert_eq!(row.field("current"), Some("true"));
        let msgs = handle(
            &mut store,
            &AppCommand::FetchSafeMode {
                session: crate::session::SessionId::UNSTAMPED,
                generation: 1,
            },
        )
        .expect("handled");
        assert!(matches!(
            msgs.as_slice(),
            [WorkerMsg::SafeModeResult {
                row: Some(row),
                ..
            }] if row.field("current") == Some("true")
        ));
    }

    #[test]
    fn demo_safe_mode_unroll_and_release_clear_hold() {
        let mut store = DemoStore::new();
        store
            .apply(&MutationOp::Command {
                endpoint: "/safe-mode".into(),
                command: "take".into(),
                fields: BTreeMap::new(),
            })
            .expect("take");
        store
            .apply(&MutationOp::Command {
                endpoint: "/safe-mode".into(),
                command: "unroll".into(),
                fields: BTreeMap::new(),
            })
            .expect("unroll");
        let row = store.rows("safe-mode").into_iter().next().expect("row");
        assert_eq!(row.field("enabled"), Some("false"));
        assert_eq!(row.field("current"), Some("false"));
        store
            .apply(&MutationOp::Command {
                endpoint: "/safe-mode".into(),
                command: "take".into(),
                fields: BTreeMap::new(),
            })
            .expect("take again");
        store
            .apply(&MutationOp::Command {
                endpoint: "/safe-mode".into(),
                command: "release".into(),
                fields: BTreeMap::new(),
            })
            .expect("release");
        let row = store.rows("safe-mode").into_iter().next().expect("row");
        assert_eq!(row.field("enabled"), Some("false"));
        assert_eq!(row.field("current"), Some("false"));
    }

    #[test]
    fn demo_disable_updates_fixture_row() {
        let mut store = DemoStore::new();
        let mut fields = BTreeMap::new();
        fields.insert(".id".into(), "*30".into());
        store
            .apply(&MutationOp::Command {
                endpoint: "/ip/firewall/filter".into(),
                command: "disable".into(),
                fields,
            })
            .expect("apply");
        let row = store
            .rows("firewall-filter")
            .into_iter()
            .find(|row| row.id == "*30")
            .expect("rule");
        assert_eq!(row.field("disabled"), Some("true"));
    }
}
