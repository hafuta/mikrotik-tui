//! Feature-owned 7.21.5 form schemas for `WireGuard` and Peers.

use crate::forms::{FieldKind, FieldSpec, FormSchema, FormSection, ScalarKind};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const LOOKUP_WG: FieldKind = FieldKind::Lookup {
    resource_id: "wireguard",
    value_key: "name",
    multiple: false,
};
const LOOKUP_VRF: FieldKind = FieldKind::Lookup {
    resource_id: "vrf",
    value_key: "name",
    multiple: false,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const MTU: FieldSpec = f!(
    "mtu",
    "MTU",
    FieldKind::ConstrainedNumber {
        min: Some(64),
        max: Some(65_535)
    }
);
const RUNNING: FieldSpec = f!("running", "Running", FieldKind::Readonly);
const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_WG);
const PUBLIC_KEY: FieldSpec = f!("public-key", "Public Key", FieldKind::Text);
const PRIVATE_KEY: FieldSpec = f!("private-key", "Private Key", FieldKind::Secret);
const LISTEN_PORT: FieldSpec = f!("listen-port", "Listen Port", FieldKind::Number);
const OPTIONAL_ENDPOINT: FieldSpec = f!(
    "endpoint-address",
    "Endpoint",
    FieldKind::Optional {
        kind: ScalarKind::Text,
        unset: "",
        unset_label: "none"
    }
);
const OPTIONAL_ENDPOINT_PORT: FieldSpec = f!(
    "endpoint-port",
    "Endpoint Port",
    FieldKind::Optional {
        kind: ScalarKind::Number {
            min: Some(0),
            max: Some(65_535)
        },
        unset: "0",
        unset_label: "none"
    }
);

pub static WIREGUARD_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["listen-port"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                LISTEN_PORT,
                MTU,
                PRIVATE_KEY,
                f!("vrf", "VRF", LOOKUP_VRF),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("public-key", "Public Key", FieldKind::Readonly), RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, LISTEN_PORT, COMMENT],
    }],
};

pub static WIREGUARD_PEER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface", "public-key"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                INTERFACE,
                PUBLIC_KEY,
                PRIVATE_KEY,
                f!("preshared-key", "Preshared Key", FieldKind::Secret),
                OPTIONAL_ENDPOINT,
                OPTIONAL_ENDPOINT_PORT,
                f!("allowed-address", "Allowed Address", FieldKind::Repeat),
                f!(
                    "persistent-keepalive",
                    "Persistent Keepalive",
                    FieldKind::Optional {
                        kind: ScalarKind::Time,
                        unset: "0s",
                        unset_label: "none"
                    }
                ),
                f!("responder", "Responder", FieldKind::Toggle),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "client",
            label: "Client",
            read_only: false,
            fields: &[
                f!("client-address", "Client Address", FieldKind::Text),
                f!("client-dns", "Client DNS", FieldKind::Repeat),
                f!("client-endpoint", "Client Endpoint", FieldKind::Text),
                f!(
                    "client-keepalive",
                    "Client Keepalive",
                    FieldKind::Optional {
                        kind: ScalarKind::Time,
                        unset: "0s",
                        unset_label: "none"
                    }
                ),
                f!(
                    "client-listen-port",
                    "Client Listen Port",
                    FieldKind::Number
                ),
                f!(
                    "client-allowed-address",
                    "Client Allowed Address",
                    FieldKind::Repeat
                ),
                f!(
                    "client-mtu",
                    "Client MTU",
                    FieldKind::ConstrainedNumber {
                        min: Some(64),
                        max: Some(65_535)
                    }
                ),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!(
                    "current-endpoint-address",
                    "Current Endpoint",
                    FieldKind::Readonly
                ),
                f!("current-endpoint-port", "Current Port", FieldKind::Readonly),
                f!("last-handshake", "Last Handshake", FieldKind::Readonly),
                f!("rx", "RX", FieldKind::Readonly),
                f!("tx", "TX", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            INTERFACE,
            PUBLIC_KEY,
            f!("allowed-address", "Allowed Address", FieldKind::Repeat),
            NAME,
            COMMENT,
        ],
    }],
};
