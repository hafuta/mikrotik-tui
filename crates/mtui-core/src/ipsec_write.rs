//! Form schemas for `/ip/ipsec` menus.
//!
//! Catalog wiring (do not register here):
//! - `ipsec-peers` → `/rest/ip/ipsec/peer`
//! - `ipsec-identities` → `/rest/ip/ipsec/identity`
//! - `ipsec-policies` → `/rest/ip/ipsec/policy`
//! - `ipsec-proposals` → `/rest/ip/ipsec/proposal`
//! - `ipsec-profiles` → `/rest/ip/ipsec/profile`
//! - `ipsec-installed-sa` → `/rest/ip/ipsec/installed-sa` (no form)
//! - `ipsec-settings` → `/rest/ip/ipsec/settings`
//! - `ipsec-key-rsa` → `/rest/ip/ipsec/key/rsa`
//! - `ipsec-key-psk` → `/rest/ip/ipsec/key/psk`
//! - `ipsec-key-qkd` → `/rest/ip/ipsec/key/qkd`
//!
//! Group id: `ip-group`.

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

const LOOKUP_PEER: FieldKind = FieldKind::Lookup {
    resource_id: "ipsec-peers",
    value_key: "name",
    multiple: false,
};
const LOOKUP_PROFILE: FieldKind = FieldKind::Lookup {
    resource_id: "ipsec-profiles",
    value_key: "name",
    multiple: false,
};
const LOOKUP_PROPOSAL: FieldKind = FieldKind::Lookup {
    resource_id: "ipsec-proposals",
    value_key: "name",
    multiple: false,
};
const LOOKUP_CERT: FieldKind = FieldKind::Lookup {
    resource_id: "certificates",
    value_key: "name",
    multiple: false,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const DISABLED: FieldSpec = f!("disabled", "Disabled", FieldKind::Toggle);
const PEER: FieldSpec = f!("peer", "Peer", LOOKUP_PEER);
const PROFILE: FieldSpec = f!("profile", "Profile", LOOKUP_PROFILE);
const PROPOSAL: FieldSpec = f!("proposal", "Proposal", LOOKUP_PROPOSAL);
const DYNAMIC: FieldSpec = f!("dynamic", "Dynamic", FieldKind::Readonly);

pub static IPSEC_PEER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["address", "profile"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                ADDRESS,
                PROFILE,
                f!("exchange-mode", "Exchange mode", FieldKind::Text),
                f!("port", "Port", FieldKind::Number),
                f!("passive", "Passive", FieldKind::Toggle),
                f!(
                    "send-initial-contact",
                    "Send initial contact",
                    FieldKind::Toggle
                ),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[
                f!("local-address", "Local address", FieldKind::Text),
                f!("responder", "Responder", FieldKind::Toggle),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                DYNAMIC,
                f!(
                    "responder-established",
                    "Responder established",
                    FieldKind::Readonly
                ),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, ADDRESS],
    }],
};

pub static IPSEC_IDENTITY_FORM: FormSchema = FormSchema {
    title_key: "peer",
    subtitle_keys: &["auth-method"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                PEER,
                f!("auth-method", "Auth method", FieldKind::Text),
                f!("secret", "Secret", FieldKind::Secret),
                f!("my-id", "My ID", FieldKind::Text),
                f!("remote-id", "Remote ID", FieldKind::Text),
                f!("certificate", "Certificate", LOOKUP_CERT),
                f!("remote-certificate", "Remote certificate", LOOKUP_CERT),
                f!("generate-policy", "Generate policy", FieldKind::Text),
                f!(
                    "policy-template-group",
                    "Policy template group",
                    FieldKind::Text
                ),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[DYNAMIC],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[PEER, f!("auth-method", "Auth method", FieldKind::Text)],
    }],
};

pub static IPSEC_POLICY_FORM: FormSchema = FormSchema {
    title_key: "src-address",
    subtitle_keys: &["dst-address", "peer"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("src-address", "Src address", FieldKind::Text),
                f!("dst-address", "Dst address", FieldKind::Text),
                f!("src-port", "Src port", FieldKind::Text),
                f!("dst-port", "Dst port", FieldKind::Text),
                f!("protocol", "Protocol", FieldKind::Text),
                f!("action", "Action", FieldKind::Text),
                f!("level", "Level", FieldKind::Text),
                f!("ipsec-protocols", "IPsec protocols", FieldKind::Text),
                PROPOSAL,
                PEER,
                f!("tunnel", "Tunnel", FieldKind::Toggle),
                f!("sa-src-address", "SA src", FieldKind::Text),
                f!("sa-dst-address", "SA dst", FieldKind::Text),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("ph2-state", "Phase 2", FieldKind::Readonly),
                DYNAMIC,
                f!("invalid", "Invalid", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("src-address", "Src address", FieldKind::Text),
            f!("dst-address", "Dst address", FieldKind::Text),
        ],
    }],
};

pub static IPSEC_PROPOSAL_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["pfs-group"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("auth-algorithms", "Auth algorithms", FieldKind::Text),
            f!("enc-algorithms", "Enc algorithms", FieldKind::Text),
            f!("pfs-group", "PFS group", FieldKind::Text),
            f!("lifetime", "Lifetime", FieldKind::Text),
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static IPSEC_PROFILE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["enc-algorithm"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("hash-algorithm", "Hash algorithm", FieldKind::Text),
            f!("enc-algorithm", "Enc algorithm", FieldKind::Text),
            f!("dh-group", "DH group", FieldKind::Text),
            f!("proposal-check", "Proposal check", FieldKind::Text),
            f!("lifetime", "Lifetime", FieldKind::Text),
            f!("nat-traversal", "NAT traversal", FieldKind::Toggle),
            f!("dpd-interval", "DPD interval", FieldKind::Text),
            f!(
                "dpd-maximum-failures",
                "DPD max failures",
                FieldKind::Number
            ),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static IPSEC_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "accounting",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("accounting", "Accounting", FieldKind::Toggle),
            f!("interim-update", "Interim update", FieldKind::Text),
            f!("xauth-use-radius", "XAuth RADIUS", FieldKind::Toggle),
            f!(
                "uniq-id-accounting",
                "Uniq-id accounting",
                FieldKind::Toggle
            ),
            f!(
                "identities-matching",
                "Identities matching",
                FieldKind::Text
            ),
        ],
    }],
    create_sections: &[],
};

pub static IPSEC_MODE_CONFIG_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["address-pool"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("address-pool", "Address pool", FieldKind::Text),
            f!("address-prefix-length", "Prefix length", FieldKind::Number),
            f!("split-include", "Split include", FieldKind::Text),
            f!("system-dns", "System DNS", FieldKind::Toggle),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static IPSEC_KEY_RSA_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["key-size"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[NAME],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("key-size", "Key size", FieldKind::Readonly)],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME],
    }],
};

pub static IPSEC_KEY_PSK_FORM: FormSchema = FormSchema {
    title_key: "peer",
    subtitle_keys: &["id"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            PEER,
            f!("id", "ID", FieldKind::Text),
            f!("key", "Key", FieldKind::Secret),
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            PEER,
            f!("id", "ID", FieldKind::Text),
            f!("key", "Key", FieldKind::Secret),
        ],
    }],
};

pub static IPSEC_KEY_QKD_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["kme-id"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                ADDRESS,
                f!("cache-size", "Cache size", FieldKind::Number),
                f!("certificate", "Certificate", LOOKUP_CERT),
                f!("key-size", "Key size", FieldKind::Number),
                f!("kme-id", "KME ID", FieldKind::Text),
                f!("peer-sae-id", "Peer SAE ID", FieldKind::Text),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("cache-state", "Cache state", FieldKind::Readonly)],
        },
    ],
    create_sections: &[],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{extra_status_fields, patch_body};
    use std::collections::HashMap;

    fn create_keys(schema: &FormSchema) -> Vec<&'static str> {
        schema
            .create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    fn status_readonly(schema: &FormSchema) {
        let status = schema
            .sections
            .iter()
            .find(|section| section.id == "status")
            .expect("status tab");
        assert!(status.read_only);
        for field in status.fields {
            assert!(!field.kind.writable(), "{}", field.key);
            assert!(!schema.writable_keys().contains(&field.key));
        }
    }

    #[test]
    fn create_peer_requires_name_and_address() {
        assert_eq!(create_keys(&IPSEC_PEER_FORM), ["name", "address"]);
        assert!(IPSEC_PEER_FORM.writable_keys().contains(&"exchange-mode"));
        assert!(IPSEC_PEER_FORM.writable_keys().contains(&"local-address"));
        assert!(!IPSEC_PEER_FORM.writable_keys().contains(&"dynamic"));
        status_readonly(&IPSEC_PEER_FORM);
    }

    #[test]
    fn identity_secret_is_masked_and_my_id_is_not() {
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("secret").map(|f| f.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("my-id").map(|f| f.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("remote-id").map(|f| f.kind),
            Some(FieldKind::Text)
        );
        assert!(IPSEC_IDENTITY_FORM.writable_keys().contains(&"peer"));
        assert!(IPSEC_IDENTITY_FORM.writable_keys().contains(&"auth-method"));
        assert!(
            IPSEC_IDENTITY_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
        status_readonly(&IPSEC_IDENTITY_FORM);
    }

    #[test]
    fn identity_patch_omits_masked_secret() {
        let mut original = HashMap::new();
        original.insert("peer".into(), "office".into());
        original.insert("secret".into(), "********".into());
        original.insert("my-id".into(), "fqdn:router.example".into());
        original.insert("comment".into(), "psk".into());
        let mut current = original.clone();
        current.insert("comment".into(), "updated".into());
        current.insert("secret".into(), "********".into());
        let body = patch_body(&IPSEC_IDENTITY_FORM, &original, &current, "********");
        assert_eq!(body.get("comment").map(String::as_str), Some("updated"));
        assert!(!body.contains_key("secret"));
        assert!(!body.contains_key("peer"));
        assert!(!body.contains_key("my-id"));
    }

    #[test]
    fn identity_patch_sends_changed_secret() {
        let mut original = HashMap::new();
        original.insert("secret".into(), "********".into());
        let mut current = original.clone();
        current.insert("secret".into(), "new-preshared-key".into());
        let body = patch_body(&IPSEC_IDENTITY_FORM, &original, &current, "********");
        assert_eq!(
            body.get("secret").map(String::as_str),
            Some("new-preshared-key")
        );
    }

    #[test]
    fn policy_status_is_readonly() {
        status_readonly(&IPSEC_POLICY_FORM);
        assert!(IPSEC_POLICY_FORM.writable_keys().contains(&"src-address"));
        assert!(IPSEC_POLICY_FORM.writable_keys().contains(&"proposal"));
        assert!(!IPSEC_POLICY_FORM.writable_keys().contains(&"ph2-state"));
        assert_eq!(
            create_keys(&IPSEC_POLICY_FORM),
            ["src-address", "dst-address"]
        );
    }

    #[test]
    fn proposal_and_profile_writable_keys() {
        for key in [
            "name",
            "auth-algorithms",
            "enc-algorithms",
            "pfs-group",
            "lifetime",
            "disabled",
        ] {
            assert!(IPSEC_PROPOSAL_FORM.writable_keys().contains(&key), "{key}");
        }
        for key in [
            "name",
            "hash-algorithm",
            "enc-algorithm",
            "dh-group",
            "proposal-check",
            "lifetime",
            "nat-traversal",
            "dpd-interval",
            "dpd-maximum-failures",
        ] {
            assert!(IPSEC_PROFILE_FORM.writable_keys().contains(&key), "{key}");
        }
        assert_eq!(create_keys(&IPSEC_PROPOSAL_FORM), ["name"]);
        assert_eq!(create_keys(&IPSEC_PROFILE_FORM), ["name"]);
    }

    #[test]
    fn settings_unknown_keys_land_on_status_extras() {
        assert!(IPSEC_SETTINGS_FORM.create_sections.is_empty());
        assert!(IPSEC_SETTINGS_FORM.writable_keys().contains(&"accounting"));
        assert!(
            IPSEC_SETTINGS_FORM
                .writable_keys()
                .contains(&"interim-update")
        );
        let mut row = HashMap::new();
        row.insert("accounting".into(), "true".into());
        row.insert("unexpected-flag".into(), "yes".into());
        let extras = extra_status_fields(&IPSEC_SETTINGS_FORM, &row);
        assert_eq!(extras, vec![("unexpected-flag".into(), "yes".into())]);
    }

    fn assert_lookup(form: &FormSchema, key: &str, resource_id: &'static str) {
        assert_eq!(
            form.field(key).map(|field| field.kind),
            Some(FieldKind::Lookup {
                resource_id,
                value_key: "name",
                multiple: false,
            }),
            "{key}"
        );
    }

    #[test]
    fn lookup_fields_point_at_named_resources() {
        assert_lookup(&IPSEC_IDENTITY_FORM, "peer", "ipsec-peers");
        assert_lookup(&IPSEC_PEER_FORM, "profile", "ipsec-profiles");
        assert_lookup(&IPSEC_POLICY_FORM, "proposal", "ipsec-proposals");
        assert_lookup(&IPSEC_IDENTITY_FORM, "certificate", "certificates");
        assert_lookup(&IPSEC_IDENTITY_FORM, "remote-certificate", "certificates");
        assert_lookup(&IPSEC_POLICY_FORM, "peer", "ipsec-peers");
    }

    #[test]
    fn non_resource_fields_stay_plain_text_or_secret() {
        assert_eq!(
            IPSEC_PEER_FORM.field("address").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("secret").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM.field("my-id").map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_IDENTITY_FORM
                .field("auth-method")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_PROPOSAL_FORM
                .field("auth-algorithms")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
        assert_eq!(
            IPSEC_PROPOSAL_FORM
                .field("enc-algorithms")
                .map(|field| field.kind),
            Some(FieldKind::Text)
        );
    }

    #[test]
    fn rsa_keys_are_named_with_readonly_size() {
        assert_eq!(create_keys(&IPSEC_KEY_RSA_FORM), ["name"]);
        assert!(IPSEC_KEY_RSA_FORM.writable_keys().contains(&"name"));
        assert!(!IPSEC_KEY_RSA_FORM.writable_keys().contains(&"key-size"));
        status_readonly(&IPSEC_KEY_RSA_FORM);
    }

    #[test]
    fn psk_keys_use_peer_lookup_and_secret() {
        assert_eq!(create_keys(&IPSEC_KEY_PSK_FORM), ["peer", "id", "key"]);
        assert_lookup(&IPSEC_KEY_PSK_FORM, "peer", "ipsec-peers");
        assert_eq!(
            IPSEC_KEY_PSK_FORM.field("key").map(|field| field.kind),
            Some(FieldKind::Secret)
        );
        assert_eq!(IPSEC_KEY_PSK_FORM.secret_keys(), ["key"]);
        assert!(
            IPSEC_KEY_PSK_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }

    #[test]
    fn qkd_is_singleton_edit_with_certificate_lookup() {
        assert!(IPSEC_KEY_QKD_FORM.create_sections.is_empty());
        assert_lookup(&IPSEC_KEY_QKD_FORM, "certificate", "certificates");
        assert!(IPSEC_KEY_QKD_FORM.writable_keys().contains(&"address"));
        assert!(IPSEC_KEY_QKD_FORM.writable_keys().contains(&"kme-id"));
        assert!(!IPSEC_KEY_QKD_FORM.writable_keys().contains(&"cache-state"));
        status_readonly(&IPSEC_KEY_QKD_FORM);
        assert!(
            IPSEC_KEY_QKD_FORM
                .sections
                .iter()
                .all(|section| section.id != "advanced")
        );
    }
}
