//! Certificate store and overlay prompt schemas.

use super::common::{CA, FILE_NAME, NAME};
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

pub static CERTIFICATE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["common-name"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("common-name", "Common Name", FieldKind::Text),
                f!("key-usage", "Key Usage", FieldKind::Text),
                f!("trusted", "Trusted", FieldKind::Toggle),
                f!("days-valid", "Days Valid", FieldKind::Number),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("digest-algorithm", "Digest", FieldKind::Readonly),
                f!("key-size", "Key Size", FieldKind::Readonly),
                f!("invalid-before", "Valid From", FieldKind::Readonly),
                f!("invalid-after", "Valid To", FieldKind::Readonly),
                f!("serial-number", "Serial", FieldKind::Readonly),
                f!("fingerprint", "Fingerprint", FieldKind::Readonly),
                f!("akid", "AKID", FieldKind::Readonly),
                f!("skid", "SKID", FieldKind::Readonly),
                f!("expires-after", "Expires", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub(crate) const CERT_EXPORT_TYPES: &[&str] = &["pem", "pkcs12"];

pub static CERT_SIGN_PROMPT: FormSchema = FormSchema {
    title_key: "ca",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "sign",
        label: "Sign",
        read_only: false,
        fields: &[CA],
    }],
};

const CERT_IMPORT_FIELDS: &[FieldSpec] = &[
    FILE_NAME,
    f!("passphrase", "Passphrase", FieldKind::Secret),
    NAME,
];

pub static CERT_IMPORT_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "import",
        label: "Import",
        read_only: false,
        fields: CERT_IMPORT_FIELDS,
    }],
};

const CERT_EXPORT_FIELDS: &[FieldSpec] = &[
    FILE_NAME,
    FieldSpec {
        key: "type",
        label: "Type",
        kind: FieldKind::Enum {
            values: CERT_EXPORT_TYPES,
        },
    },
    f!("export-passphrase", "Export Passphrase", FieldKind::Secret),
];

pub static CERT_EXPORT_PROMPT: FormSchema = FormSchema {
    title_key: "file-name",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "export",
        label: "Export",
        read_only: false,
        fields: CERT_EXPORT_FIELDS,
    }],
};
