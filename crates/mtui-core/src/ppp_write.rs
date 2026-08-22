//! Form schemas for the `PPP` nav group.

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
const USER: FieldSpec = f!("user", "User", FieldKind::Text);
const PASSWORD: FieldSpec = f!("password", "Password", FieldKind::Secret);
const PROFILE: FieldSpec = f!("profile", "Profile", FieldKind::Text);
const RUNNING: FieldSpec = f!("running", "Running", FieldKind::Readonly);
const INTERFACE: FieldSpec = f!("interface", "Interface", FieldKind::Text);
const CONNECT_TO: FieldSpec = f!("connect-to", "Connect to", FieldKind::Text);
const ADD_DEFAULT_ROUTE: FieldSpec = f!("add-default-route", "Default", FieldKind::Toggle);
const DEFAULT_PROFILE: FieldSpec = f!("default-profile", "Profile", FieldKind::Text);
const AUTHENTICATION: FieldSpec = f!("authentication", "Auth", FieldKind::Text);
const KEEPALIVE: FieldSpec = f!("keepalive-timeout", "Keepalive", FieldKind::Text);
const MAX_MTU: FieldSpec = f!("max-mtu", "Max MTU", FieldKind::Number);
const MAX_MRU: FieldSpec = f!("max-mru", "Max MRU", FieldKind::Number);
const ENABLED: FieldSpec = f!("enabled", "Enabled", FieldKind::Toggle);
const CERTIFICATE: FieldSpec = f!("certificate", "Certificate", FieldKind::Text);
const USE_IPSEC: FieldSpec = f!("use-ipsec", "IPsec", FieldKind::Toggle);
const IPSEC_SECRET: FieldSpec = f!("ipsec-secret", "IPsec secret", FieldKind::Secret);
const SERVICE_NAME: FieldSpec = f!("service-name", "Service", FieldKind::Text);
const MODE: FieldSpec = f!("mode", "Mode", FieldKind::Text);
const AUTH: FieldSpec = f!("auth", "Auth", FieldKind::Text);
const CIPHER: FieldSpec = f!("cipher", "Cipher", FieldKind::Text);

pub static PPP_SECRET_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["service", "profile"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            PASSWORD,
            f!("service", "Service", FieldKind::Text),
            PROFILE,
            f!("caller-id", "Caller", FieldKind::Text),
            f!("local-address", "Local", FieldKind::Text),
            f!("remote-address", "Remote", FieldKind::Text),
            f!("remote-ipv6-prefix", "IPv6 prefix", FieldKind::Text),
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            PASSWORD,
            f!("service", "Service", FieldKind::Text),
            PROFILE,
            COMMENT,
        ],
    }],
};

pub static PPP_PROFILE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("local-address", "Local", FieldKind::Text),
            f!("remote-address", "Remote", FieldKind::Text),
            f!("dns-server", "DNS", FieldKind::Text),
            f!("rate-limit", "Rate limit", FieldKind::Text),
            f!("only-one", "Only one", FieldKind::Toggle),
            f!("use-encryption", "Encrypt", FieldKind::Text),
            f!("use-compression", "Compress", FieldKind::Text),
            f!("change-tcp-mss", "MSS", FieldKind::Text),
            f!("bridge", "Bridge", FieldKind::Text),
            f!("interface-list", "List", FieldKind::Text),
            COMMENT,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, COMMENT],
    }],
};

pub static PPP_AAA_FORM: FormSchema = FormSchema {
    title_key: "use-radius",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("use-radius", "RADIUS", FieldKind::Toggle),
            f!("accounting", "Accounting", FieldKind::Toggle),
            f!("interim-update", "Interim", FieldKind::Text),
            f!("enable-ipv6-accounting", "IPv6 acct", FieldKind::Toggle),
        ],
    }],
    create_sections: &[],
};

pub static PPP_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["port"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("port", "Port", FieldKind::Text),
                USER,
                PASSWORD,
                PROFILE,
                f!("phone", "Phone", FieldKind::Text),
                ADD_DEFAULT_ROUTE,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("port", "Port", FieldKind::Text),
            USER,
            PASSWORD,
            COMMENT,
        ],
    }],
};

pub static PPPOE_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                INTERFACE,
                USER,
                PASSWORD,
                SERVICE_NAME,
                f!("ac-name", "AC name", FieldKind::Text),
                PROFILE,
                ADD_DEFAULT_ROUTE,
                f!("use-peer-dns", "Peer DNS", FieldKind::Toggle),
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[f!("status", "Status", FieldKind::Readonly), RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, INTERFACE, USER, PASSWORD],
    }],
};

pub static PPPOE_SERVER_FORM: FormSchema = FormSchema {
    title_key: "service-name",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            SERVICE_NAME,
            INTERFACE,
            DEFAULT_PROFILE,
            AUTHENTICATION,
            MAX_MTU,
            MAX_MRU,
            f!("one-session-per-host", "One sess", FieldKind::Toggle),
            f!("max-sessions", "Max sess", FieldKind::Number),
            KEEPALIVE,
            COMMENT,
            DISABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[SERVICE_NAME, INTERFACE],
    }],
};

pub static PPTP_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["connect-to"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                CONNECT_TO,
                USER,
                PASSWORD,
                PROFILE,
                ADD_DEFAULT_ROUTE,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, CONNECT_TO, USER, PASSWORD],
    }],
};

pub static PPTP_SERVER_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["default-profile"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ENABLED,
            DEFAULT_PROFILE,
            AUTHENTICATION,
            KEEPALIVE,
            MAX_MTU,
            MAX_MRU,
            f!("mrru", "MRRU", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static L2TP_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["connect-to"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                CONNECT_TO,
                USER,
                PASSWORD,
                PROFILE,
                USE_IPSEC,
                IPSEC_SECRET,
                ADD_DEFAULT_ROUTE,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, CONNECT_TO, USER, PASSWORD],
    }],
};

pub static L2TP_SERVER_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["default-profile"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ENABLED,
            DEFAULT_PROFILE,
            AUTHENTICATION,
            USE_IPSEC,
            IPSEC_SECRET,
            KEEPALIVE,
            MAX_MTU,
            MAX_MRU,
            f!("allow-fast-path", "Fast path", FieldKind::Toggle),
        ],
    }],
    create_sections: &[],
};

pub static SSTP_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["connect-to"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                CONNECT_TO,
                USER,
                PASSWORD,
                PROFILE,
                CERTIFICATE,
                f!("verify-server-certificate", "Verify", FieldKind::Toggle),
                ADD_DEFAULT_ROUTE,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, CONNECT_TO, USER, PASSWORD],
    }],
};

pub static SSTP_SERVER_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["certificate"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ENABLED,
            CERTIFICATE,
            DEFAULT_PROFILE,
            AUTHENTICATION,
            f!("port", "Port", FieldKind::Number),
            f!("verify-client-certificate", "Verify", FieldKind::Toggle),
            KEEPALIVE,
            MAX_MTU,
            MAX_MRU,
        ],
    }],
    create_sections: &[],
};

pub static OVPN_CLIENT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["connect-to"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                CONNECT_TO,
                f!("port", "Port", FieldKind::Number),
                MODE,
                USER,
                PASSWORD,
                PROFILE,
                CERTIFICATE,
                CIPHER,
                AUTH,
                ADD_DEFAULT_ROUTE,
                COMMENT,
                DISABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[RUNNING],
        },
    ],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, CONNECT_TO, USER, PASSWORD],
    }],
};

pub static OVPN_SERVER_FORM: FormSchema = FormSchema {
    title_key: "enabled",
    subtitle_keys: &["port"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            ENABLED,
            f!("port", "Port", FieldKind::Number),
            MODE,
            f!("netmask", "Netmask", FieldKind::Text),
            CERTIFICATE,
            DEFAULT_PROFILE,
            AUTH,
            CIPHER,
            f!(
                "require-client-certificate",
                "Client cert",
                FieldKind::Toggle
            ),
        ],
    }],
    create_sections: &[],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{extra_status_fields, patch_body};
    use std::collections::HashMap;

    const FORMS: &[&FormSchema] = &[
        &PPP_SECRET_FORM,
        &PPP_PROFILE_FORM,
        &PPP_AAA_FORM,
        &PPP_CLIENT_FORM,
        &PPPOE_CLIENT_FORM,
        &PPPOE_SERVER_FORM,
        &PPTP_CLIENT_FORM,
        &PPTP_SERVER_FORM,
        &L2TP_CLIENT_FORM,
        &L2TP_SERVER_FORM,
        &SSTP_CLIENT_FORM,
        &SSTP_SERVER_FORM,
        &OVPN_CLIENT_FORM,
        &OVPN_SERVER_FORM,
    ];

    fn create_keys(form: &FormSchema) -> Vec<&'static str> {
        form.create_sections
            .iter()
            .flat_map(|section| section.fields.iter().map(|field| field.key))
            .collect()
    }

    #[test]
    fn writable_keys_include_secret_fields() {
        for form in [
            &PPP_SECRET_FORM,
            &PPP_CLIENT_FORM,
            &PPPOE_CLIENT_FORM,
            &PPTP_CLIENT_FORM,
            &L2TP_CLIENT_FORM,
            &SSTP_CLIENT_FORM,
            &OVPN_CLIENT_FORM,
        ] {
            assert!(form.writable_keys().contains(&"password"));
            assert_eq!(
                form.field("password").map(|field| field.kind),
                Some(FieldKind::Secret)
            );
        }
        for form in [&L2TP_CLIENT_FORM, &L2TP_SERVER_FORM] {
            assert!(form.writable_keys().contains(&"ipsec-secret"));
            assert_eq!(
                form.field("ipsec-secret").map(|field| field.kind),
                Some(FieldKind::Secret)
            );
        }
        assert!(!PPP_CLIENT_FORM.writable_keys().contains(&"running"));
        assert!(!PPPOE_CLIENT_FORM.writable_keys().contains(&"status"));
    }

    #[test]
    fn patch_body_omits_masked_password() {
        let mut original = HashMap::new();
        original.insert("name".into(), "user1".into());
        original.insert("password".into(), "********".into());
        original.insert("profile".into(), "default".into());
        let mut current = original.clone();
        current.insert("profile".into(), "office".into());
        current.insert("password".into(), "********".into());
        let body = patch_body(&PPP_SECRET_FORM, &original, &current, "********");
        assert_eq!(body.get("profile").map(String::as_str), Some("office"));
        assert!(!body.contains_key("password"));
    }

    #[test]
    fn patch_body_omits_masked_ipsec_secret() {
        let mut original = HashMap::new();
        original.insert("name".into(), "l2tp1".into());
        original.insert("ipsec-secret".into(), "********".into());
        original.insert("connect-to".into(), "1.1.1.1".into());
        let mut current = original.clone();
        current.insert("connect-to".into(), "8.8.8.8".into());
        current.insert("ipsec-secret".into(), "********".into());
        let body = patch_body(&L2TP_CLIENT_FORM, &original, &current, "********");
        assert_eq!(body.get("connect-to").map(String::as_str), Some("8.8.8.8"));
        assert!(!body.contains_key("ipsec-secret"));
        assert!(!body.contains_key("running"));
    }

    #[test]
    fn create_sections_are_short() {
        assert_eq!(
            create_keys(&PPP_SECRET_FORM),
            ["name", "password", "service", "profile", "comment"]
        );
        assert_eq!(create_keys(&PPP_PROFILE_FORM), ["name", "comment"]);
        assert!(PPP_AAA_FORM.create_sections.is_empty());
        assert_eq!(
            create_keys(&PPP_CLIENT_FORM),
            ["name", "port", "user", "password", "comment"]
        );
        assert_eq!(
            create_keys(&PPPOE_CLIENT_FORM),
            ["name", "interface", "user", "password"]
        );
        assert_eq!(
            create_keys(&PPPOE_SERVER_FORM),
            ["service-name", "interface"]
        );
        assert_eq!(
            create_keys(&PPTP_CLIENT_FORM),
            ["name", "connect-to", "user", "password"]
        );
        assert!(PPTP_SERVER_FORM.create_sections.is_empty());
        assert_eq!(
            create_keys(&L2TP_CLIENT_FORM),
            ["name", "connect-to", "user", "password"]
        );
        assert!(L2TP_SERVER_FORM.create_sections.is_empty());
        assert_eq!(
            create_keys(&SSTP_CLIENT_FORM),
            ["name", "connect-to", "user", "password"]
        );
        assert!(SSTP_SERVER_FORM.create_sections.is_empty());
        assert_eq!(
            create_keys(&OVPN_CLIENT_FORM),
            ["name", "connect-to", "user", "password"]
        );
        assert!(OVPN_SERVER_FORM.create_sections.is_empty());
        for form in FORMS {
            assert!(create_keys(form).len() <= 5);
        }
    }

    #[test]
    fn no_empty_advanced_tabs() {
        for form in FORMS {
            assert!(
                form.sections
                    .iter()
                    .all(|section| section.id != "advanced" || !section.fields.is_empty())
            );
            assert!(form.sections.iter().all(|section| section.id != "advanced"));
        }
    }

    #[test]
    fn unknown_keys_land_on_status_extras() {
        let mut row = HashMap::new();
        row.insert("name".into(), "pppoe1".into());
        row.insert("dynamic".into(), "true".into());
        let extras = extra_status_fields(&PPPOE_CLIENT_FORM, &row);
        assert_eq!(extras, vec![("dynamic".into(), "true".into())]);
    }
}
