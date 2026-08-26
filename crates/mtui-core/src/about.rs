//! On-demand screen guides for `RouterOS` menus.
//!
//! Copy is original wording aligned with
//! <https://manual.mikrotik.com/docs/>. Property tables are not reproduced.
//! Every catalog id must have an entry; the CLI reference URL is derived from
//! the resource path so field names stay tied to what `RouterOS` exposes.

use crate::resources::{DASHBOARD_ID, resource_by_id};

const CLI_DOCS: &str = "https://manual.mikrotik.com/docs/cli-reference";
const MANUAL: &str = "https://manual.mikrotik.com/docs";

/// Section heading for the operator-facing “do I need this?” copy.
pub const WHEN_YOU_NEED_IT: &str = "When you need it";

/// Curated explanation for one navigation screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenGuide {
    pub summary: &'static str,
    pub use_when: &'static str,
    pub fields: &'static str,
    /// Conceptual manual page when one exists; CLI reference is always added.
    pub docs_url: Option<&'static str>,
}

/// Title, path kicker, and wrapped body for the about overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutCopy {
    pub title: String,
    pub kicker: String,
    pub body: String,
}

/// Guide for the open screen (`dashboard` or a resource id).
#[must_use]
pub fn screen_guide(id: &str) -> Option<&'static ScreenGuide> {
    crate::features::interfaces::guides::GUIDES
        .iter()
        .chain(crate::features::wireguard::guides::GUIDES)
        .chain(crate::features::ppp::guides::GUIDES)
        .chain(crate::features::bridge::guides::GUIDES)
        .chain(crate::features::switch::guides::GUIDES)
        .chain(crate::features::ip::guides::GUIDES)
        .chain(crate::features::ipv6::guides::GUIDES)
        .chain(GUIDES)
        .find(|(key, _)| *key == id)
        .map(|(_, guide)| guide)
}

/// Formatted overlay copy, or `None` when the id is unknown.
#[must_use]
pub fn about_copy(id: &str) -> Option<AboutCopy> {
    let guide = screen_guide(id)?;
    let (title, kicker, cli_url) = if id == DASHBOARD_ID {
        (
            "About Dashboard".to_string(),
            "overview".to_string(),
            format!("{MANUAL}/introduction/"),
        )
    } else {
        let spec = resource_by_id(id)?;
        (
            format!("About {}", spec.label),
            spec.cli_path().to_string(),
            format!("{CLI_DOCS}{}/", spec.cli_path()),
        )
    };

    let mut body = String::new();
    body.push_str(guide.summary);
    body.push_str("\n\n");
    body.push_str(WHEN_YOU_NEED_IT);
    body.push('\n');
    body.push_str(guide.use_when);
    if !guide.fields.is_empty() {
        body.push_str("\n\nNotable fields\n");
        body.push_str(guide.fields);
    }
    body.push_str("\n\nOfficial documentation\n");
    if let Some(url) = guide.docs_url
        && url != cli_url
    {
        body.push_str(url);
        body.push('\n');
    }
    body.push_str(&cli_url);
    Some(AboutCopy {
        title,
        kicker,
        body,
    })
}

macro_rules! guide {
    ($id:literal, $summary:literal, $when:literal, $fields:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: None,
            },
        )
    };
    ($id:literal, $summary:literal, $when:literal, $fields:literal, $docs:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: Some($docs),
            },
        )
    };
}

static GUIDES: &[(&str, ScreenGuide)] = &[
    guide!(
        "dashboard",
        "Live CPU, memory, WAN throughput, and firewall-hit overview for the connected \
         router. This is an mtui dashboard, not a RouterOS menu.",
        "Use it as a first look at whether the device is busy or the WAN is moving traffic. \
         Open a specific menu when you need to change configuration.",
        "Charts are sampled locally. Firewall rows are hit counters from filter rules, not \
         a rule editor.",
        "https://manual.mikrotik.com/docs/introduction/"
    ),
    guide!(
        "containers",
        "Linux containers on RouterOS v7. Images come from a registry (remote-image) or a \
         tar already on Files. Adding a row starts download or extract; it does not start \
         the container. Status, arch, OS, and tag are what the device stored after extract.",
        "Needs the container extra package (arm, arm64, x86, CHR). Device-mode container=yes \
         needs a reset or mode button, or a cold power-off on x86, within the timeout. DNS \
         must be set on IP DNS or on the container. EN7562CT boards only run arm32v5 images; \
         the registry rejects other architectures. This client does not filter image names.",
        "name, interface (VETH), remote-image or file, root-dir, envlist, mountlists, \
         start-on-boot, logging, memory limits, healthcheck (7.23+), status/arch/tag.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-config",
        "Global container settings: registry URL, extract directory, layer store, and \
         registry username/password.",
        "Set registry-url and tmpdir on disk before a remote-image add. Password is stored \
         on the router; this client masks it in the sheet.",
        "registry-url, tmpdir, layer-dir, username, password, memory-high/max, swap-max, \
         assumed-registry-url, memory-current.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-envs",
        "Named environment lists. Each row is a list name plus one key and value. A \
         container points at a list with envlist.",
        "Group variables per app. RouterOS does not mark env values as secrets.",
        "list, key, value.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-mounts",
        "Named bind-mount lists. Each row is list, host src, and path inside the container. \
         Containers reference lists with mountlists.",
        "Point src at a disk path that already exists on the router.",
        "list, src, dst.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "apps",
        "MikroTik app catalog on top of containers. YAML plus NAT and veth that RouterOS \
         applies. arm64 and x86 only; EN7562CT is not supported for Apps even when \
         containers are.",
        "Use it when you want a packaged app instead of a raw container row. The device \
         fetches the catalog; this client lists /app.",
        "name, network (internal/lan/default), YAML, environment/mounts/redirects, status, \
         UI URL, IP.",
        "https://manual.mikrotik.com/docs/containers/apps/"
    ),
    guide!(
        "routing-tables",
        "Named routing tables (including main and VRF FIBs).",
        "Extra tables are for policy routing. fib marks whether the table installs in the FIB.",
        "name, fib, dynamic."
    ),
    guide!(
        "routing-rules",
        "Policy routing rules: select a table from src/dst/routing-mark before looking up \
         the main table.",
        "Use with mangle routing marks or multi-WAN. Order matters.",
        "src-address, dst-address, routing-mark, action, table, disabled."
    ),
    guide!(
        "ospf-instances",
        "OSPF routing instances (v2/v3): router-id and how default routes are originated.",
        "Need dynamic IGP inside an AS. Areas and interface templates are sibling menus.",
        "name, version, router-id, originate-default, disabled."
    ),
    guide!(
        "ospf-areas",
        "OSPF areas belonging to an instance: area-id and type (backbone, stub, NSSA, …).",
        "Split a large domain. Area 0.0.0.0 is backbone. Attach networks via interface \
         templates.",
        "name, instance, area-id, type, disabled."
    ),
    guide!(
        "ospf-interface-templates",
        "OSPF interface templates (RouterOS v7): which interfaces sit in which area.",
        "Bind instance and area to one or more interfaces. Live cost and adjacency state \
         are on OSPF Interface, not on this template.",
        "instance, area, interfaces, type, disabled."
    ),
    guide!(
        "ospf-interfaces",
        "Live OSPF interfaces after templates match: address, area, state, cost, and DR/BDR.",
        "Watch interface state and metric. Change cost or network type on OSPF Interface \
         Templates. Monitor-only; there is no Add.",
        "address, area, state, network-type, cost, dr, bdr."
    ),
    guide!(
        "bgp-connections",
        "BGP connections (RouterOS v7 style): remote address/AS and local role.",
        "Peering with ISPs or other ASes. Templates and address-families may exist beyond \
         this table.",
        "name, remote.address, remote.as, local.role, disabled."
    ),
    guide!(
        "bgp-templates",
        "Reusable BGP session defaults (AS, router-id, address-families) for connections.",
        "Put common peering options on a template, then point connections at it.",
        "name, as, router-id, address-families, output.network, disabled."
    ),
    guide!(
        "queue-simple",
        "Simple queues: easy per-target (IP/interface) rate limits using HTB.",
        "Cap a customer or a WAN. For hierarchical sharing use Queue Tree plus packet marks.",
        "name, target, max-limit, burst, packet-marks, parent, disabled."
    ),
    guide!(
        "queue-tree",
        "HTB queue tree: hierarchical classes usually keyed by packet-mark.",
        "Build QoS after mangle marks traffic. Needs a parent (often global-in/out or an \
         interface).",
        "name, parent, packet-mark, max-limit, limit-at, priority, bucket-size."
    ),
    guide!(
        "queue-type",
        "Queue type definitions: pfifo, sfq, pcq, fq-codel, cake, and so on.",
        "Reuse a type from simple/tree queues. PCQ is common for per-peer fairness.",
        "name, kind, and kind-specific options (pcq-rate, fq-codel-limit, …)."
    ),
    guide!(
        "queue-interface",
        "Default queue type attached to an interface’s software queue.",
        "Change only if you know you need a specific qdisc on that port.",
        "interface, queue (type name)."
    ),
    guide!(
        "files",
        "Router filesystem: backups, scripts, images, and uploaded files.",
        "Save a named backup or load a `.backup` file from the action menu (that replaces the \
         running configuration and reboots). Pull a file onto the router with /tool/fetch (`f`). \
         Removing a file here deletes it on the router. Local contents upload/download is not \
         available over the classic API.",
        "name, type, size, creation-time. Contents are not shown in the table."
    ),
    guide!(
        "netwatch",
        "Host monitoring: ICMP/TCP/HTTP probes that can run on-up/on-down scripts.",
        "Watch a gateway or extra WAN and trigger failover scripts. Not a replacement for \
         routing protocols.",
        "host, type, interval, timeout, status, up/down-script, disabled."
    ),
    guide!(
        "email",
        "SMTP client used by `/tool e-mail` and some scripts.",
        "Configure if you want the router to send alerts. Password is a secret.",
        "server, port, from, user, password, tls (yes/starttls/no)."
    ),
    guide!(
        "romon",
        "Router Management Overlay Network: an independent L2 overlay used to reach neighbors \
         when IP routing is down. Packets use EtherType 0x88bf and are not shown in sniffer or torch.",
        "Enable it when you need out-of-band management through a neighbor. Secrets authenticate \
         frames with MD5 hashing; they are not encryption. Use SSH or a secure WinBox session on \
         top. A zero ID lets the router pick current-id from a port MAC.",
        "enabled, id, secrets (list), current-id (runtime).",
        "https://manual.mikrotik.com/docs/management-tools/romon/"
    ),
    guide!(
        "romon-ports",
        "Which interfaces (or interface lists) take part in RoMON, with a cost and optional \
         per-port secrets. A default all entry is present on typical routers.",
        "Restrict RoMON to backbone ports, raise cost on slower links, or forbid a WAN. \
         Port secrets override the global list when they are set.",
        "interface (all or a name), forbid, cost, secrets, disabled, comment.",
        "https://manual.mikrotik.com/docs/management-tools/romon/"
    ),
    guide!(
        "graphing",
        "Built-in RouterOS graphs for CPU/memory/disk, interface traffic, and simple queues. \
         Collection is configured here; the pictures are served at /graphs/ on the router HTTP(S) \
         service, not as local dashboard sparks.",
        "Turn graphing on when you want history on the box. Avoid store-on-disk on devices with \
         tiny flash. Page refresh may be seconds or never.",
        "store-every (5min, hour, 24hours), page-refresh.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
    ),
    guide!(
        "graphing-interface",
        "Which interfaces graphing samples for traffic charts, and which addresses may view them.",
        "Add all or a named interface. Tighten allow-address on untrusted networks.",
        "interface, allow-address, store-on-disk, disabled, comment.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
    ),
    guide!(
        "graphing-queue",
        "Which simple queues graphing samples, plus whether the queue target-address may view charts.",
        "Use it for per-queue history. If a queue target is 0.0.0.0/0, allow-target can open graphs \
         more widely than allow-address.",
        "simple-queue, allow-address, allow-target, store-on-disk, disabled, comment.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
    ),
    guide!(
        "graphing-resource",
        "CPU, memory, and disk usage graphs. There is no per-interface selector on this list.",
        "Add an entry so resource charts exist, then restrict allow-address if the graphs page \
         should not be public.",
        "allow-address, store-on-disk, disabled, comment.",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
    ),
    guide!(
        "ping",
        "One-shot ICMP (or similar) reachability check from the router to an address.",
        "Confirm a host is reachable from this router, not from your workstation. Replies stream \
         until the count finishes or you close the overlay.",
        "address (required), count, src-address. Results appear in the Ping overlay; this \
         screen is not a live poll of /tool/ping."
    ),
    guide!(
        "traceroute",
        "Hop-by-hop path discovery from the router toward an address.",
        "See where packets leave this device on the way to a destination. Hop replies stream \
         until the probe finishes or you close the overlay.",
        "address (required), src-address, protocol (icmp by default), count/max hops. Open \
         the overlay with Enter; t is reserved for interface torch elsewhere."
    ),
    guide!(
        "radius",
        "RADIUS clients: where to send AAA for login, PPP, Hotspot, DHCP, wireless, and \
         similar services.",
        "Add your RADIUS server and enable the matching service. Incoming RADIUS (if used) \
         is a related system setting.",
        "address, protocol, secret, service, timeout, src-address, disabled."
    ),
    guide!(
        "users",
        "RouterOS login accounts (full, write, read, group-based).",
        "Create operators. Prefer groups over sharing admin. Passwords are secrets.",
        "name, group, address, inactivity-policy, inactivity-timeout, last-logged-in, disabled."
    ),
    guide!(
        "special-login",
        "Serial-port proxy logins: an SSH/Telnet user is bound to a `/port` instead of the RouterOS CLI.",
        "Use it so a dedicated account drops straight onto a serial device. Disable the matching `/system/console` binding first or the port stays owned by the local console.",
        "user, port, disabled.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328139/Serial+Console"
    ),
    guide!(
        "user-groups",
        "Permission groups for local users (and some AAA mappings).",
        "Define what a role may read or write instead of using the built-in full user for \
         everything.",
        "name, policy flags, skin."
    ),
    guide!(
        "routerboard",
        "Hardware identity: model, serial, firmware, and factory settings (RouterBOOT).",
        "Read-only inventory. Firmware upgrades are a different, careful operation.",
        "model, serial-number, firmware-type, current/upgrade-firmware, board-name. Upgrade and USB power reset are actions.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328068/RouterBOARD"
    ),
    guide!(
        "routerboard-settings",
        "RouterBOOT settings: boot device, OS, frequencies, and protected RouterBOOT.",
        "Change boot order or silent-boot on hardware that exposes `/system/routerboard/settings`. Missing on CHR.",
        "boot-os, boot-device, boot-protocol, cpu-frequency, protected-routerboot, silent-boot, auto-upgrade.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328068/RouterBOARD"
    ),
    guide!(
        "routerboard-mode-button",
        "Mode-button script: hold time and the script to run.",
        "Wire a physical mode button to a `/system script` on boards that have `/system/routerboard/mode-button`.",
        "enabled, hold-time, on-event.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328068/RouterBOARD"
    ),
    guide!(
        "routerboard-reset-button",
        "Reset-button script: hold time and the script to run.",
        "Same idea as the mode button, on boards that expose `/system/routerboard/reset-button`.",
        "enabled, hold-time, on-event.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328068/RouterBOARD"
    ),
    guide!(
        "ntp",
        "NTP client: how this router synchronizes its clock from NTP servers.",
        "Point the client at reliable NTP sources. Certificates, logs, and many services need a \
         sane clock.",
        "enabled, servers, mode, status.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/40992869/NTP"
    ),
    guide!(
        "ntp-server",
        "NTP server: this router as an NTP source for LAN clients.",
        "Enable the server so clients can unicast to the router. Broadcast needs \
         broadcast-addresses. Set local-clock-stratum when use-local-clock is on.",
        "enabled, broadcast, multicast, manycast, broadcast-addresses, vrf, use-local-clock, \
         local-clock-stratum, auth-key.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/40992869/NTP"
    ),
    guide!(
        "ntp-keys",
        "NTP symmetric keys: numeric key ids and their secret values.",
        "Create a key here, then pick its id as Auth. Key on NTP Server (or leave none).",
        "key-id, key-val.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/40992869/NTP"
    ),
    guide!(
        "clock",
        "Local date, time, and time zone.",
        "Set zone even when NTP is on, so logs print local time.",
        "time, date, time-zone-name, gmt-offset."
    ),
    guide!(
        "license",
        "RouterOS license status for this device: Software ID and nlevel on RouterBOARD or x86, \
         System ID and CHR level on Cloud Hosted Router.",
        "Check the level before an upgrade or a CHR move. Apply a key or import a file already on \
         the router; this client never prints or logs a license key. Output-key is not offered.",
        "software-id, nlevel, features, expires-in on hardware. system-id, level, next-renewal-at, \
         deadline-at, limited-upgrades on CHR.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328149/RouterOS+license+keys"
    ),
    guide!(
        "disks",
        "Attached storage: USB, NAND, RAID, tmpfs, and network-backed slots under `/disk`.",
        "Inspect size and filesystem before containers or extra logging. Format and eject ask for \
         confirmation. RAID type and role are sheet fields with a save preview; they are not silent \
         extra commands.",
        "slot, type, mount-filesystem, RAID type/role/master, size, free, fs, state. Format needs \
         a file-system type.",
        "https://manual.mikrotik.com/docs/hardware/disks/"
    ),
    guide!(
        "device-mode",
        "RouterOS v7 device-mode: which features (container, scheduler, traffic-gen, fetch, and \
         others) this box is allowed to run. Home, basic, advanced, and ROSE presets each leave \
         some flags off until you enable them.",
        "Read the flags before blaming a missing menu. Saving here sends `/system/device-mode \
         update`, not a silent PATCH. RouterOS then waits for a reset or mode button press, or a \
         cold power-off, within the activation timeout (default 5 minutes). The device reboots when \
         the change is confirmed. If you do nothing, the update is canceled.",
        "mode, per-feature yes/no flags, flagged, flagging-enabled, allowed-versions, attempt-count.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/93749258/Device-mode"
    ),
    guide!(
        "identity",
        "System identity string shown in neighbors, WinBox, and prompts.",
        "Set a unique name per device. It is not a DNS name unless you also create DNS.",
        "name."
    ),
    guide!(
        "resources",
        "CPU, memory, HDD, uptime, version — `/system resource`.",
        "Capacity and version checks. Reboot or shut down the router from this screen; those \
         commands are system-wide, not operations on the resource table itself.",
        "uptime, version, cpu, cpu-load, free/total-memory, architecture."
    ),
    guide!(
        "health",
        "Hardware sensors: voltage, temperature, fans where the board has them.",
        "Spot PSU or thermal issues. Not every model exports health.",
        "name, value, type — varies by hardware."
    ),
    guide!(
        "packages",
        "Installed RouterOS packages (wireless, extra, …) and their versions.",
        "See what is enabled. Installing packages is a reboot-class change; do it with a \
         plan.",
        "name, version, build-time, scheduled, disabled."
    ),
    guide!(
        "scheduler",
        "Scheduled scripts: run a `/system script` at intervals or calendar times.",
        "Automate backups or housekeeping. The script body lives under Scripts.",
        "name, start-date/time, interval, on-event, disabled."
    ),
    guide!(
        "scripts",
        "Stored RouterOS scripts (the source you schedule or run on events).",
        "Keep automation here rather than one-off terminal history. Policy/permissions \
         apply when they run.",
        "name, owner, policy, source (long), dont-require-permissions."
    ),
    guide!(
        "logging",
        "Log rules: which topics go to an action (memory, disk, echo, email, or remote syslog).",
        "Tune noise vs audit. Each rule picks an action from Logging Actions. The Logs screen \
         is the memory/file tail, not this config.",
        "topics, action, prefix, disabled."
    ),
    guide!(
        "logging-actions",
        "Log destinations: memory, disk, console echo, remote syslog, email, or a script. \
         Built-in names (memory, disk, echo, remote, email) exist on typical routers.",
        "Configure the destination here, then point a Logging rule at it. An action is unused \
         until a rule uses its name. Fields follow Type: memory, disk, echo, remote, email, or \
         script. Remote syslog adds address, port, protocol (udp, tcp, or tls), format, and VRF. \
         Check Certificate appears only for TLS. Syslog Facility and Syslog Severity appear only \
         for BSD syslog; CEF Event Delimiter only for CEF.",
        "name, Type, then fields for the selected Type (memory lines, disk file, remote syslog, \
         email, or script).",
        "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/log/"
    ),
    guide!(
        "snmp",
        "SNMP agent: enable, contact, location, trap targets.",
        "For NMS polling. Use v3 where you can; communities are secrets of a sort.",
        "enabled, contact, location, trap-version, trap-community, src-address."
    ),
    guide!(
        "snmp-communities",
        "SNMPv1/v2c communities (and v3 users depending on version).",
        "Restrict addresses; do not use public/private on the internet.",
        "name, addresses, read-access, write-access, security."
    ),
    guide!(
        "certificates",
        "Local certificate store: CA, device certs, CSRs for www-ssl, SSTP, IPsec, OpenVPN. \
         Create an empty request, sign against a CA (or the same name for a root), import a \
         file already on the router, or export PEM/PKCS12.",
        "Needed for api-ssl/WinBox TLS and several VPN types. Keys and passphrases stay \
         secret. Sign with g, import with p, export with w.",
        "name, common-name, key-usage, ca, file-name, type, passphrase, export-passphrase."
    ),
    guide!(
        "watchdog",
        "Hardware/software watchdog: reboot if the system stops pinging a target or hangs.",
        "Safety net on remote sites. A bad watch-address can reboot-loop the box. Fields edit in place; Ctrl+s patches.",
        "watch-address, watchdog-timer, watch-interval, no-ping-delay, ping-timeout, automatic-supout."
    ),
    guide!(
        "system-console",
        "Serial console bindings (`/system/console`): attach a local terminal to a `/port`.",
        "Not the in-app log pane. Disabling the last serial console can lock you out of that port.",
        "port, term, channel, disabled. Runtime used/free/wedged stay on Status.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328139/Serial+Console"
    ),
    guide!(
        "leds",
        "Per-LED bindings (`/system/led`): type, interface or modem, and which LEDs light.",
        "Map board LEDs to link or modem activity. LED Settings is the sibling singleton for all-off.",
        "type, interface, modem, leds, disabled.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/8978532/LEDs"
    ),
    guide!(
        "led-settings",
        "Board-wide LED settings (`/system/led/settings`).",
        "Turn every LED off immediately, after an hour, or never. Separate from per-LED bindings.",
        "all-leds-off.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/8978532/LEDs"
    ),
    guide!(
        "ports",
        "Serial port hardware (`/port`): baud, parity, and flow control.",
        "Console and Special Login look up these names. This is not the interactive serial terminal.",
        "name, baud-rate, data-bits, parity, stop-bits, flow-control.",
        "https://help.mikrotik.com/docs/spaces/ROS/pages/328139/Serial+Console"
    ),
    guide!(
        "note",
        "Administrative note shown on login (banner-like text).",
        "Leave a contact or change warning for the next operator.",
        "note text, show-at-login."
    ),
    guide!(
        "package-update",
        "Package update channel and check/install (`/system package update`).",
        "Check for a new RouterOS build, then install (reboot-class). Distinct from RouterBOARD firmware.",
        "channel, installed-version, latest-version, status. Check and Install are actions."
    ),
    guide!(
        "reset-configuration",
        "Factory-style `/system reset-configuration` with flags on the page, then a confirm.",
        "Set keep-users, no-defaults, skip-backup, caps-mode, and run-after-reset here. Ctrl+s asks before POST. Destructive; never Safe Mode.",
        "keep-users, no-defaults, skip-backup, caps-mode, run-after-reset."
    ),
    guide!(
        "reboot",
        "Reboot this router (`/system reboot`).",
        "Opens a confirm as soon as you select the System item. Esc cancels without POST.",
        "No fields. Same warning as Resources used to show when Safe Mode is on."
    ),
    guide!(
        "shutdown",
        "Power off this router (`/system shutdown`).",
        "Opens a confirm as soon as you select the System item. Esc cancels without POST.",
        "No fields. Same warning as Resources used to show when Safe Mode is on."
    ),
    guide!(
        "ssh-keys",
        "User SSH public keys (`/user ssh-keys`).",
        "Install keys so operators can log in without a password. Private keys stay off this table unless the API exposes them.",
        "user, key-owner."
    ),
    guide!(
        "history",
        "Configuration history (`/system history`). Undo a selected row after a confirm prompt.",
        "See who changed what locally. Undo runs `/system history undo` for that row. It is not Safe Mode unroll; take or release Safe Mode with F4. Rows tagged F (floating-undo) are Safe Mode work that unrolls if that session dies.",
        "floating-undo, time, action, by, policy. Undo is a row action."
    ),
    guide!(
        "rip-instances",
        "RIP instance (`/routing rip instance`) for RIP v1/v2 on ROS 7.",
        "Only if you still speak RIP with a neighbor. Prefer OSPF/BGP otherwise.",
        "name, vrf, originate-default, disabled."
    ),
    guide!(
        "rip-interface-templates",
        "RIP interface templates (which interfaces run RIP).",
        "Attach an instance to interfaces. Look up the instance name.",
        "instance, interfaces, disabled."
    ),
    guide!(
        "bfd",
        "BFD sessions/configuration (`/routing bfd configuration`).",
        "Faster neighbor failure detection for OSPF/BGP. Easy to flap a link if timers are too tight.",
        "interfaces, addresses, min-tx-interval, min-rx-interval, multiplier."
    ),
    guide!(
        "routing-filters",
        "ROS 7 routing filters (`/routing filter rule`) — the large chain/rule language.",
        "Control what OSPF/BGP accept or advertise. The rule body is a script-like filter.",
        "chain, rule, disabled, comment."
    ),
    guide!(
        "routing-id",
        "Routing IDs (`/routing id`) used by OSPF/BGP instances.",
        "Set a stable router-id selector instead of relying on a random address.",
        "name, id, select, disabled."
    ),
    guide!(
        "ospf-neighbors",
        "OSPF neighbor table. Monitor-only; no Add.",
        "See adjacency state. Configure instances/areas/templates elsewhere.",
        "instance, router-id, address, state, adjacency."
    ),
    guide!(
        "ospf-lsa",
        "OSPF LSA database. Monitor-only.",
        "Inspect what the area flooded. Not an editor.",
        "type, id, originator, area, sequence."
    ),
    guide!(
        "bgp-advertisements",
        "BGP advertisements table. Monitor-only.",
        "See what this router is announcing. VPN table is omitted unless the API lists it stably.",
        "prefix, nexthop, peer, as-path."
    ),
    guide!(
        "sniffer",
        "Packet sniffer start/stop, optional save-to-file on the router.",
        "Capture on-box; there is no live pcap UI. Start/stop are actions.",
        "interface, file-name, file-limit, filter-stream, filter-interface."
    ),
    guide!(
        "bandwidth-test",
        "Bandwidth-test overlay (client to a MikroTik bandwidth-test server).",
        "Measure throughput from this router. No graphs — streamed samples in the overlay.",
        "address, protocol, duration/count, direction/user if the server requires them."
    ),
    guide!(
        "flood-ping",
        "Flood-ping overlay for a burst of ICMP from the router.",
        "Stress a path briefly. Close the overlay to stop.",
        "address, count, src-address."
    ),
    guide!(
        "mac-scan",
        "MAC-scan overlay on a L2 interface.",
        "Discover neighbors by MAC on a LAN segment.",
        "interface (src), results: address/mac-address/age."
    ),
    guide!(
        "ip-scan",
        "IP-scan overlay for a range on an interface.",
        "Find which addresses answer on a subnet.",
        "address range, interface (src), mac-address, time."
    ),
    guide!(
        "profiler",
        "CPU profiler overlay (`/tool profile`).",
        "See which processes burn CPU. No WebFig-style graphs.",
        "samples of name, usage, load."
    ),
    guide!(
        "wol",
        "Wake-on-LAN one-shot (`/tool wol`).",
        "Send a magic packet out an interface to a MAC.",
        "interface, mac."
    ),
    guide!(
        "sms",
        "SMS send (`/tool sms send`) when the sms package exists.",
        "Send a short message via a modem channel. Skip if the package is absent.",
        "phone-number, message, channel."
    ),
    guide!(
        "radius-incoming",
        "RADIUS incoming (`/radius incoming`) — accept incoming RADIUS on a port.",
        "Needed for some disconnect/CoA setups. Not User Manager.",
        "accept, port."
    ),
    guide!(
        "logs",
        "Live log tail from `/log` (topics + message), newest first. This client keeps a \
         bounded local buffer; it does not delete logs on the router when you clear the view.",
        "Debug events and errors. Configure what is recorded under Logging.",
        "time, topics, message. Space pauses the view; severity filter is local."
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ALL_RESOURCES;

    #[test]
    fn every_resource_and_dashboard_has_a_guide() {
        let mut ids: Vec<&str> = crate::features::interfaces::guides::GUIDES
            .iter()
            .chain(crate::features::wireguard::guides::GUIDES)
            .chain(crate::features::ppp::guides::GUIDES)
            .chain(crate::features::bridge::guides::GUIDES)
            .chain(crate::features::switch::guides::GUIDES)
            .chain(crate::features::ip::guides::GUIDES)
            .chain(crate::features::ipv6::guides::GUIDES)
            .chain(GUIDES)
            .map(|(id, _)| *id)
            .collect();
        let original = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), original, "duplicate screen guide ids");
        assert!(screen_guide(DASHBOARD_ID).is_some());
        for spec in ALL_RESOURCES.iter() {
            assert!(
                screen_guide(spec.id).is_some(),
                "missing screen guide for {}",
                spec.id
            );
        }
    }

    #[test]
    fn neighbors_guide_mentions_connect_tab() {
        let guide = screen_guide("neighbors").expect("neighbors");
        assert!(guide.use_when.contains("Connect"), "{}", guide.use_when);
        assert!(guide.use_when.contains("device tab"), "{}", guide.use_when);
        let copy = about_copy("neighbors").expect("copy");
        assert!(copy.kicker.contains("/ip/neighbor"));
        assert!(!copy.body.contains('\u{2014}'));
    }

    #[test]
    fn macsec_guide_tracks_the_manual() {
        let guide = screen_guide("macsec").expect("macsec");
        let copy = about_copy("macsec").expect("copy");
        let hay = format!("{} {} {}", guide.summary, guide.use_when, guide.fields);
        for needle in ["802.1AE", "GCM-AES-128", "CAK", "CKN", "Dot1x", "Ethernet"] {
            assert!(hay.contains(needle), "missing {needle}");
        }
        assert!(copy.body.contains("manual.mikrotik.com"));
        assert!(copy.body.contains("/interface/macsec"));
        assert_eq!(
            guide.docs_url,
            Some("https://manual.mikrotik.com/docs/bridging-and-switching/macsec/")
        );
        assert!(copy.kicker.contains("/interface/macsec"));
        assert!(
            !copy.body.to_ascii_lowercase().contains("paraphrased"),
            "about copy must not mention paraphrasing"
        );
    }

    #[test]
    fn lte_apn_guide_points_at_the_cli_reference() {
        let guide = screen_guide("lte-apn").expect("lte-apn");
        let copy = about_copy("lte-apn").expect("copy");
        let hay = format!("{} {} {}", guide.summary, guide.use_when, guide.fields);
        for needle in ["APN", "authentication", "use-network-apn"] {
            assert!(hay.contains(needle), "missing {needle}");
        }
        assert!(copy.kicker.contains("/interface/lte/apn"));
        assert!(copy.body.contains("/interface/lte/apn"));
        assert!(copy.body.contains("manual.mikrotik.com"));
        assert!(
            !copy.body.to_ascii_lowercase().contains("paraphrased"),
            "about copy must not mention paraphrasing"
        );
        assert!(!copy.body.contains('\u{2014}'));
    }

    #[test]
    fn cli_docs_url_follows_the_resource_path() {
        let copy = about_copy("vlan").expect("vlan");
        assert!(
            copy.body
                .contains("https://manual.mikrotik.com/docs/cli-reference/interface/vlan/")
        );
        assert_eq!(copy.title, "About VLAN");
    }

    #[test]
    fn interface_list_guides_cross_link_definitions_and_members() {
        let lists = about_copy("interface-lists").expect("lists");
        let members = about_copy("interface-list-members").expect("members");
        assert_eq!(lists.title, "About Lists");
        assert_eq!(members.title, "About List members");
        assert!(lists.body.contains("List members"));
        assert!(members.body.contains("Lists"));
        assert!(lists.body.to_ascii_lowercase().contains("include"));
        assert!(lists.body.to_ascii_lowercase().contains("exclude"));
        assert!(members.body.contains("join") || members.body.contains("Joins"));
    }

    #[test]
    fn ipv6_firewall_connections_guide_mirrors_ipv4() {
        let guide = screen_guide("ipv6-firewall-connections").expect("guide");
        let copy = about_copy("ipv6-firewall-connections").expect("copy");
        assert_eq!(copy.title, "About Connections");
        assert!(copy.kicker.contains("/ipv6/firewall/connection"));
        assert!(copy.body.contains("/ipv6/firewall/connection"));
        assert!(guide.summary.to_ascii_lowercase().contains("ipv6"));
        assert!(guide.use_when.to_ascii_lowercase().contains("remove"));
        assert!(guide.fields.contains("src/dst-address"));
        assert_eq!(
            guide.docs_url,
            Some(
                "https://manual.mikrotik.com/docs/firewall-and-quality-of-service/connection-tracking/"
            )
        );
        assert!(
            !copy.body.contains('\u{2014}'),
            "about copy must not use em dashes"
        );
    }

    #[test]
    fn ospf_interface_guide_is_runtime_not_a_template() {
        let live = about_copy("ospf-interfaces").expect("ospf-interfaces");
        let templates = about_copy("ospf-interface-templates").expect("templates");
        assert_eq!(live.title, "About OSPF Interface");
        assert_eq!(templates.title, "About OSPF Interface Templates");
        assert!(live.kicker.contains("/routing/ospf/interface"));
        assert!(!live.kicker.contains("interface-template"));
        assert!(templates.kicker.contains("interface-template"));
        assert!(live.body.contains("Monitor-only"));
        assert!(live.body.contains("cost"));
        assert!(templates.body.contains("OSPF Interface"));
        assert!(!templates.body.contains("no separate"));
        assert!(
            live.body
                .contains("https://manual.mikrotik.com/docs/cli-reference/routing/ospf/interface/")
        );
    }

    #[test]
    fn traffic_flow_and_igmp_guides_track_the_manual() {
        let flow = about_copy("traffic-flow").expect("traffic-flow");
        assert_eq!(flow.title, "About Traffic Flow");
        assert!(flow.kicker.contains("/ip/traffic-flow"));
        assert!(flow.body.contains("NetFlow") || flow.body.contains("IPFIX"));
        assert!(flow.body.contains("Packet Sampling"));
        assert!(!flow.body.contains('\u{2014}'));

        let targets = about_copy("traffic-flow-targets").expect("targets");
        assert!(targets.body.contains("Dst. Address"));
        assert!(targets.body.contains("ipfix"));

        let proxy = about_copy("igmp-proxy").expect("igmp-proxy");
        assert!(proxy.kicker.contains("/routing/igmp-proxy"));
        assert!(proxy.body.contains("upstream"));
        assert!(!proxy.body.contains('\u{2014}'));

        let ifaces = about_copy("igmp-proxy-interfaces").expect("ifaces");
        assert!(ifaces.body.contains("Upstream"));
        let mfc = about_copy("igmp-proxy-mfc").expect("mfc");
        assert!(mfc.body.contains("Group"));
    }

    #[test]
    fn romon_and_graphing_guides_track_the_manual() {
        let romon = about_copy("romon").expect("romon");
        let ports = about_copy("romon-ports").expect("ports");
        let graphing = about_copy("graphing").expect("graphing");
        assert_eq!(romon.title, "About RoMON");
        assert_eq!(ports.title, "About RoMON Ports");
        assert_eq!(graphing.title, "About Graphing");
        assert!(romon.kicker.contains("/tool/romon"));
        assert!(ports.kicker.contains("/tool/romon/port"));
        assert!(graphing.kicker.contains("/tool/graphing"));
        assert!(romon.body.to_ascii_lowercase().contains("secret"));
        assert!(ports.body.contains("forbid"));
        assert!(graphing.body.contains("/graphs/"));
        assert!(
            about_copy("graphing-interface")
                .expect("gi")
                .kicker
                .contains("/tool/graphing/interface")
        );
        assert!(
            about_copy("graphing-queue")
                .expect("gq")
                .kicker
                .contains("/tool/graphing/queue")
        );
        assert!(
            about_copy("graphing-resource")
                .expect("gr")
                .kicker
                .contains("/tool/graphing/resource")
        );
        for copy in [&romon, &ports, &graphing] {
            assert!(
                !copy.body.contains('\u{2014}'),
                "about copy must not use em dashes"
            );
        }
        assert_eq!(
            screen_guide("romon").expect("g").docs_url,
            Some("https://manual.mikrotik.com/docs/management-tools/romon/")
        );
        assert_eq!(
            screen_guide("graphing").expect("g").docs_url,
            Some(
                "https://manual.mikrotik.com/docs/diagnostics-monitoring-and-troubleshooting/graphing/"
            )
        );
    }

    #[test]
    fn history_guide_covers_undo_and_keeps_safe_mode_separate() {
        let copy = about_copy("history").expect("history");
        assert_eq!(copy.title, "About History");
        assert!(copy.kicker.contains("/system/history"));
        assert!(copy.body.contains("undo"));
        assert!(copy.body.contains("F4"));
        assert!(copy.body.contains("Safe Mode"));
        assert!(copy.body.contains("floating-undo"));
    }
}
