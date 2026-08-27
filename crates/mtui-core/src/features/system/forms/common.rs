//! Shared lookups and field specs for System forms.

use crate::forms::{FieldKind, FieldSpec};

pub(crate) const LOOKUP_USER_GROUP: FieldKind = FieldKind::Lookup {
    resource_id: "user-groups",
    value_key: "name",
    multiple: false,
};
pub(crate) const LOOKUP_USER: FieldKind = FieldKind::Lookup {
    resource_id: "users",
    value_key: "name",
    multiple: false,
};
pub(crate) const LOOKUP_SCRIPT: FieldKind = FieldKind::Lookup {
    resource_id: "scripts",
    value_key: "name",
    multiple: false,
};
pub(crate) const LOOKUP_CERTIFICATE: FieldKind = FieldKind::Lookup {
    resource_id: "certificates",
    value_key: "name",
    multiple: false,
};
pub(crate) const LOOKUP_FILE: FieldKind = FieldKind::Lookup {
    resource_id: "files",
    value_key: "name",
    multiple: false,
};
pub(crate) const LOOKUP_VRF: FieldKind = FieldKind::Lookup {
    resource_id: "vrf",
    value_key: "name",
    multiple: false,
};
pub(crate) const LOOKUP_NTP_KEY: FieldKind = FieldKind::Lookup {
    resource_id: "ntp-keys",
    value_key: "key-id",
    multiple: false,
};
pub(crate) const LOOKUP_DISK: FieldKind = FieldKind::Lookup {
    resource_id: "disks",
    value_key: "slot",
    multiple: false,
};
pub(crate) const LOOKUP_IFACE: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
pub(crate) const LOOKUP_PORT: FieldKind = FieldKind::Lookup {
    resource_id: "ports",
    value_key: "name",
    multiple: false,
};

pub(crate) const NAME: FieldSpec = FieldSpec {
    key: "name",
    label: "Name",
    kind: FieldKind::Text,
};
pub(crate) const COMMENT: FieldSpec = FieldSpec {
    key: "comment",
    label: "Comment",
    kind: FieldKind::Text,
};
pub(crate) const ENABLED: FieldSpec = FieldSpec {
    key: "disabled",
    label: "Enabled",
    kind: FieldKind::InvertedToggle,
};
pub(crate) const PASSWORD: FieldSpec = FieldSpec {
    key: "password",
    label: "Password",
    kind: FieldKind::Secret,
};
pub(crate) const SOURCE: FieldSpec = FieldSpec {
    key: "source",
    label: "Source",
    kind: FieldKind::Text,
};
pub(crate) const GROUP: FieldSpec = FieldSpec {
    key: "group",
    label: "Group",
    kind: LOOKUP_USER_GROUP,
};
pub(crate) const OWNER: FieldSpec = FieldSpec {
    key: "owner",
    label: "Owner",
    kind: FieldKind::Readonly,
};
pub(crate) const ON_EVENT: FieldSpec = FieldSpec {
    key: "on-event",
    label: "On event",
    kind: LOOKUP_SCRIPT,
};
pub(crate) const POLICY: FieldSpec = FieldSpec {
    key: "policy",
    label: "Policy",
    kind: FieldKind::Text,
};
pub(crate) const CA: FieldSpec = FieldSpec {
    key: "ca",
    label: "CA",
    kind: LOOKUP_CERTIFICATE,
};
pub(crate) const FILE_NAME: FieldSpec = FieldSpec {
    key: "file-name",
    label: "File Name",
    kind: LOOKUP_FILE,
};
