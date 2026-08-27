//! Feature-owned operator guides for System screens.

use crate::about::ScreenGuide;

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

pub(crate) static GUIDES: &[(&str, ScreenGuide)] = &[
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
        "auto-upgrade, boot-device, boot-os, boot-protocol, cpu-frequency, enable-jumper-reset, silent-boot, protected-routerboot.",
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
        "user-groups",
        "Permission groups for local users (and some AAA mappings).",
        "Define what a role may read or write instead of using the built-in full user for \
         everything.",
        "name, policy flags, skin."
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
        "note",
        "Administrative note shown on login (banner-like text).",
        "Leave a contact or change warning for the next operator.",
        "note text, show-at-login."
    ),
    guide!(
        "logs",
        "Live log tail from `/log` (topics + message), newest first. This client keeps a \
         bounded local buffer; it does not delete logs on the router when you clear the view.",
        "Debug events and errors. Configure what is recorded under Logging.",
        "time, topics, message. Space pauses the view; severity filter is local."
    ),
];
