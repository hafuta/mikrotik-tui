//! Prompt schemas for Files actions. The Files table has no entity sheet;
//! Upload and Download open as centered modals over the listing.

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

const LOCAL_PATH: FieldSpec = f!("local-path", "Local Path", FieldKind::Text);
const REMOTE_NAME: FieldSpec = f!("remote-name", "Remote Name", FieldKind::Text);

const UPLOAD_SECTIONS: &[FormSection] = &[FormSection {
    id: "general",
    label: "General",
    read_only: false,
    fields: &[LOCAL_PATH, REMOTE_NAME],
}];

const DOWNLOAD_SECTIONS: &[FormSection] = &[FormSection {
    id: "general",
    label: "General",
    read_only: false,
    fields: &[LOCAL_PATH],
}];

/// Workstation → router. Enter on Local Path opens the directory browser.
pub static UPLOAD_FORM: FormSchema = FormSchema {
    title_key: "remote-name",
    subtitle_keys: &[],
    sections: UPLOAD_SECTIONS,
    create_sections: &[],
};

/// Router → workstation. Enter on Local Path opens the directory browser.
pub static DOWNLOAD_FORM: FormSchema = FormSchema {
    title_key: "local-path",
    subtitle_keys: &[],
    sections: DOWNLOAD_SECTIONS,
    create_sections: &[],
};
