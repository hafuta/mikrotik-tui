//! Form schemas for the RADIUS navigation group.
//!
//! Catalog wiring (do not register here):
//! - `radius` → `/radius` (`RADIUS_FORM`)
//! - `radius-incoming` → `/radius/incoming` (`RADIUS_INCOMING_FORM`)
//!
//! Group id: `radius-group`.

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

const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Text);
const SECRET: FieldSpec = f!("secret", "Secret", FieldKind::Secret);
const SERVICE: FieldSpec = f!("service", "Service", FieldKind::Repeat);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const PROTOCOL: FieldSpec = f!(
    "protocol",
    "Protocol",
    FieldKind::Enum {
        values: &["udp", "tcp", "radsec"],
    }
);

pub static RADIUS_FORM: FormSchema = FormSchema {
    title_key: "address",
    subtitle_keys: &["service"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                ADDRESS,
                PROTOCOL,
                SECRET,
                SERVICE,
                f!(
                    "authentication-port",
                    "Authentication Port",
                    FieldKind::Number
                ),
                f!("accounting-port", "Accounting Port", FieldKind::Number),
                f!("timeout", "Timeout", FieldKind::Time),
                f!("src-address", "Src. Address", FieldKind::Ip),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("dynamic", "Dynamic", FieldKind::Readonly),
                f!("invalid", "Invalid", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static RADIUS_INCOMING_FORM: FormSchema = FormSchema {
    title_key: "accept",
    subtitle_keys: &["port"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("accept", "Accept", FieldKind::Toggle),
            f!("port", "Port", FieldKind::Number),
        ],
    }],
    create_sections: &[],
};
