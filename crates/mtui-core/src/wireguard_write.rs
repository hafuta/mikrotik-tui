//! Form schemas for the `WireGuard` nav group.

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
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const MTU: FieldSpec = f!("mtu", "MTU", FieldKind::Number);
const RUNNING: FieldSpec = f!("running", "Running", FieldKind::Readonly);
const INTERFACE: FieldSpec = f!("interface", "Interface", FieldKind::Text);
const PUBLIC_KEY: FieldSpec = f!("public-key", "Public key", FieldKind::Text);
const PRIVATE_KEY: FieldSpec = f!("private-key", "Private key", FieldKind::Secret);

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
                f!("listen-port", "Listen port", FieldKind::Number),
                MTU,
                PRIVATE_KEY,
                f!("vrf", "VRF", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("public-key", "Public key", FieldKind::Readonly), RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("listen-port", "Listen port", FieldKind::Number),
            COMMENT,
        ],
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
                f!("preshared-key", "Preshared key", FieldKind::Secret),
                f!("endpoint-address", "Endpoint", FieldKind::Text),
                f!("endpoint-port", "Endpoint port", FieldKind::Number),
                f!("allowed-address", "Allowed address", FieldKind::Text),
                f!("persistent-keepalive", "Keepalive", FieldKind::Text),
                f!("responder", "Responder", FieldKind::Toggle),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "client",
            label: "Client",
            read_only: false,
            fields: &[
                f!("client-address", "Client address", FieldKind::Text),
                f!("client-dns", "Client DNS", FieldKind::Text),
                f!("client-endpoint", "Client endpoint", FieldKind::Text),
                f!("client-keepalive", "Client keepalive", FieldKind::Text),
                f!("client-listen-port", "Client listen", FieldKind::Number),
                f!("client-allowed-address", "Client allowed", FieldKind::Text),
                f!("client-mtu", "Client MTU", FieldKind::Number),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!(
                    "current-endpoint-address",
                    "Current endpoint",
                    FieldKind::Readonly
                ),
                f!("current-endpoint-port", "Current port", FieldKind::Readonly),
                f!("last-handshake", "Last handshake", FieldKind::Readonly),
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
            f!("allowed-address", "Allowed address", FieldKind::Text),
            NAME,
            COMMENT,
        ],
    }],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{extra_status_fields, patch_body};
    use std::collections::HashMap;

    #[test]
    fn wireguard_edit_matches_winbox_properties() {
        assert_eq!(
            WIREGUARD_FORM.writable_keys(),
            [
                "name",
                "listen-port",
                "mtu",
                "private-key",
                "vrf",
                "comment",
                "disabled",
            ]
        );
        assert_eq!(
            WIREGUARD_FORM.known_keys(),
            [
                "name",
                "listen-port",
                "mtu",
                "private-key",
                "vrf",
                "comment",
                "disabled",
                "public-key",
                "running",
            ]
        );
        let create: Vec<_> = WIREGUARD_FORM
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect();
        assert_eq!(create, ["name", "listen-port", "comment"]);
        assert!(
            WIREGUARD_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }

    #[test]
    fn peer_form_has_client_tab_and_short_create() {
        let tabs: Vec<_> = WIREGUARD_PEER_FORM
            .sections
            .iter()
            .map(|section| section.id)
            .collect();
        assert_eq!(tabs, ["general", "client", "status"]);
        assert!(
            WIREGUARD_PEER_FORM
                .sections
                .iter()
                .find(|section| section.id == "status")
                .is_some_and(|section| section.read_only)
        );
        let create: Vec<_> = WIREGUARD_PEER_FORM
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect();
        assert_eq!(
            create,
            [
                "interface",
                "public-key",
                "allowed-address",
                "name",
                "comment"
            ]
        );
        assert!(
            WIREGUARD_PEER_FORM
                .writable_keys()
                .contains(&"preshared-key")
        );
        assert!(WIREGUARD_PEER_FORM.writable_keys().contains(&"client-mtu"));
        assert!(!WIREGUARD_PEER_FORM.writable_keys().contains(&"rx"));
    }

    #[test]
    fn patch_body_keeps_masked_private_key() {
        let mut original = HashMap::new();
        original.insert("name".into(), "wg1".into());
        original.insert("private-key".into(), "********".into());
        original.insert("listen-port".into(), "13231".into());
        let mut current = original.clone();
        current.insert("listen-port".into(), "51820".into());
        current.insert("private-key".into(), "********".into());
        let body = patch_body(&WIREGUARD_FORM, &original, &current, "********");
        assert_eq!(body.get("listen-port").map(String::as_str), Some("51820"));
        assert!(!body.contains_key("private-key"));
        assert!(!body.contains_key("public-key"));
    }

    #[test]
    fn unknown_peer_keys_land_on_status_extras() {
        let mut row = HashMap::new();
        row.insert("interface".into(), "wg1".into());
        row.insert("dynamic".into(), "true".into());
        let extras = extra_status_fields(&WIREGUARD_PEER_FORM, &row);
        assert_eq!(extras, vec![("dynamic".into(), "true".into())]);
    }
}
