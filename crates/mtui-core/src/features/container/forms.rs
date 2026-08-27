//! Feature-owned form schemas for the `Container` navigation group.

use crate::forms::{FieldKind, FieldSpec, FormSchema, FormSection};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);

const LOOKUP_VETH: FieldKind = FieldKind::Lookup {
    resource_id: "veth",
    value_key: "name",
    multiple: false,
};
const LOOKUP_FILE: FieldKind = FieldKind::Lookup {
    resource_id: "files",
    value_key: "name",
    multiple: false,
};
const LOOKUP_ENV_LIST: FieldKind = FieldKind::Lookup {
    resource_id: "container-envs",
    value_key: "list",
    multiple: false,
};
const LOOKUP_MOUNT_LISTS: FieldKind = FieldKind::Lookup {
    resource_id: "container-mounts",
    value_key: "list",
    multiple: true,
};

pub const RESTART_POLICY: &[&str] = &["no", "on-failure", "always"];
pub const APP_NETWORK: &[&str] = &["internal", "lan", "default"];

pub static CONTAINER_CONFIG_FORM: FormSchema = FormSchema {
    title_key: "registry-url",
    subtitle_keys: &["tmpdir"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("registry-url", "Registry URL", FieldKind::Text),
                f!("tmpdir", "Tmp Dir", FieldKind::Text),
                f!("layer-dir", "Layer Dir", FieldKind::Text),
                f!("username", "Username", FieldKind::Text),
                f!("password", "Password", FieldKind::Secret),
                f!("memory-high", "Memory High", FieldKind::Number),
                f!("memory-max", "Memory Max", FieldKind::Number),
                f!("swap-max", "Swap Max", FieldKind::Number),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!(
                    "assumed-registry-url",
                    "Assumed Registry URL",
                    FieldKind::Readonly
                ),
                f!("memory-current", "Memory Current", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static CONTAINER_ENV_FORM: FormSchema = FormSchema {
    title_key: "list",
    subtitle_keys: &["key"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("list", "List", FieldKind::Text),
            f!("key", "Key", FieldKind::Text),
            f!("value", "Value", FieldKind::Text),
            COMMENT,
        ],
    }],
    create_sections: &[],
};

pub static CONTAINER_MOUNT_FORM: FormSchema = FormSchema {
    title_key: "list",
    subtitle_keys: &["dst"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("list", "List", FieldKind::Text),
            f!("src", "Src", FieldKind::Text),
            f!("dst", "Dst", FieldKind::Text),
            COMMENT,
        ],
    }],
    create_sections: &[],
};

pub static CONTAINER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["status", "tag"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                COMMENT,
                f!("interface", "Interface", LOOKUP_VETH),
                f!("remote-image", "Remote Image", FieldKind::Text),
                f!("file", "File", LOOKUP_FILE),
                f!("root-dir", "Root Dir", FieldKind::Text),
                f!("layer-dir", "Layer Dir", FieldKind::Text),
                f!("envlist", "Env List", LOOKUP_ENV_LIST),
                f!("mountlists", "Mount Lists", LOOKUP_MOUNT_LISTS),
                f!("start-on-boot", "Start On Boot", FieldKind::Toggle),
                f!("logging", "Logging", FieldKind::Toggle),
                f!("dns", "DNS", FieldKind::Text),
                f!("domain-name", "Domain Name", FieldKind::Text),
                f!("hostname", "Hostname", FieldKind::Text),
                f!("workdir", "Work Dir", FieldKind::Text),
                f!("cmd", "Cmd", FieldKind::Text),
                f!("entrypoint", "Entrypoint", FieldKind::Text),
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!(
                    "restart-policy",
                    "Restart Policy",
                    FieldKind::Enum {
                        values: RESTART_POLICY,
                    }
                ),
                f!("restart-interval", "Restart Interval", FieldKind::Text),
                f!("restart-max-count", "Restart Max Count", FieldKind::Number),
                f!("stop-signal", "Stop Signal", FieldKind::Text),
                f!("stop-time", "Stop Time", FieldKind::Text),
                f!("user", "User", FieldKind::Text),
                f!("devices", "Devices", FieldKind::Text),
                f!("cpu-list", "CPU List", FieldKind::Text),
                f!("memory-high", "Memory High", FieldKind::Number),
                f!("memory-max", "Memory Max", FieldKind::Number),
                f!("swap-max", "Swap Max", FieldKind::Number),
                f!("shm-size", "Shm Size", FieldKind::Number),
                f!("tmpfs", "Tmpfs", FieldKind::Text),
                f!("hosts", "Hosts", FieldKind::Repeat),
                f!("mount", "Mount", FieldKind::Text),
                f!("stop-on-unhealthy", "Stop On Unhealthy", FieldKind::Toggle),
                f!("healthcheck-cmd", "Healthcheck Cmd", FieldKind::Text),
                f!(
                    "healthcheck-interval",
                    "Healthcheck Interval",
                    FieldKind::Text
                ),
                f!(
                    "healthcheck-retries",
                    "Healthcheck Retries",
                    FieldKind::Number
                ),
                f!(
                    "healthcheck-start-interval",
                    "Healthcheck Start Interval",
                    FieldKind::Text
                ),
                f!(
                    "healthcheck-start-period",
                    "Healthcheck Start Period",
                    FieldKind::Text
                ),
                f!(
                    "healthcheck-timeout",
                    "Healthcheck Timeout",
                    FieldKind::Text
                ),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("status", "Status", FieldKind::Readonly),
                f!("arch", "Arch", FieldKind::Readonly),
                f!("os", "OS", FieldKind::Readonly),
                f!("tag", "Tag", FieldKind::Readonly),
                f!("memory-current", "Memory Current", FieldKind::Readonly),
                f!("cpu-usage", "CPU Usage", FieldKind::Readonly),
                f!(
                    "healthcheck-status",
                    "Healthcheck Status",
                    FieldKind::Readonly
                ),
            ],
        },
    ],
    create_sections: &[],
};

pub static APP_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["status"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("auto-update", "Auto Update", FieldKind::Toggle),
                f!("check-certificate", "Check Certificate", FieldKind::Toggle),
                f!(
                    "network",
                    "Network",
                    FieldKind::Enum {
                        values: APP_NETWORK,
                    }
                ),
                f!(
                    "network-outgoing-access",
                    "Network Outgoing Access",
                    FieldKind::Toggle
                ),
                f!("use-https", "Use HTTPS", FieldKind::Toggle),
                f!("pvid", "PVID", FieldKind::Number),
                f!(
                    "container-command-lines",
                    "Container Command Lines",
                    FieldKind::Text
                ),
                f!("devices", "Devices", FieldKind::Text),
                f!("environment", "Environment", FieldKind::Repeat),
                f!("extra-mounts", "Extra Mounts", FieldKind::Repeat),
                f!(
                    "firewall-redirects",
                    "Firewall Redirects",
                    FieldKind::Repeat
                ),
                f!("yaml", "YAML", FieldKind::Text),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("status", "Status", FieldKind::Readonly),
                f!("running", "Running", FieldKind::Readonly),
                f!("ui-url", "UI URL", FieldKind::Readonly),
                f!("ip-address", "IP Address", FieldKind::Readonly),
                f!("interface", "Interface", FieldKind::Readonly),
                f!("app-size", "App Size", FieldKind::Readonly),
                f!("data-size", "Data Size", FieldKind::Readonly),
                f!("memory-current", "Memory Current", FieldKind::Readonly),
                f!("cpu-usage", "CPU Usage", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};
