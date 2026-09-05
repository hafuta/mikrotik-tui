//! Form schemas for `/ip/hotspot` and `/ip/proxy`.

use crate::form_fields::{
    FIELD_ALLOW_DENY, FIELD_PROXY_ACCESS_ACTION, KIND_HOTSPOT_BINDING_TYPE, KIND_HTTP_METHOD,
    LOOKUP_ADDRESS_LIST, LOOKUP_INTERFACES, LOOKUP_POOLS,
};
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

const LOOKUP_PROFILE: FieldKind = FieldKind::Lookup {
    resource_id: "hotspot-profiles",
    value_key: "name",
    multiple: false,
};
const LOOKUP_USER_PROFILE: FieldKind = FieldKind::Lookup {
    resource_id: "hotspot-user-profiles",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SERVER: FieldKind = FieldKind::Lookup {
    resource_id: "hotspot",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SCRIPT: FieldKind = FieldKind::Lookup {
    resource_id: "scripts",
    value_key: "name",
    multiple: false,
};

const OPEN_STATUS_PAGE: &[&str] = &["always", "http-login"];
const OPTIONAL_TIME_NONE: FieldKind = FieldKind::Optional {
    kind: ScalarKind::Time,
    unset: "none",
    unset_label: "none",
};
const OPTIONAL_SHARED_USERS: FieldKind = FieldKind::Optional {
    kind: ScalarKind::Number {
        min: None,
        max: None,
    },
    unset: "unlimited",
    unset_label: "unlimited",
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_INTERFACES);
const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Ip);
const MAC: FieldSpec = f!("mac-address", "MAC address", FieldKind::Mac);

pub static HOTSPOT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["interface", "profile"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            INTERFACE,
            f!("address-pool", "Address pool", LOOKUP_POOLS),
            f!("profile", "Profile", LOOKUP_PROFILE),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static HOTSPOT_PROFILE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["hotspot-address"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("hotspot-address", "Hotspot address", FieldKind::Ip),
            f!("dns-name", "DNS name", FieldKind::Text),
            f!("html-directory", "HTML directory", FieldKind::Text),
            f!("login-by", "Login by", FieldKind::Repeat),
            f!("use-radius", "Use RADIUS", FieldKind::Toggle),
            COMMENT,
        ],
    }],
    create_sections: &[],
};

pub static HOTSPOT_USER_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["profile", "server"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("password", "Password", FieldKind::Secret),
            f!("profile", "Profile", LOOKUP_USER_PROFILE),
            f!("server", "Server", LOOKUP_SERVER),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static HOTSPOT_USER_PROFILE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["rate-limit", "shared-users"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("session-timeout", "Session Timeout", FieldKind::Time),
                f!("idle-timeout", "Idle Timeout", OPTIONAL_TIME_NONE),
                f!("keepalive-timeout", "Keepalive Timeout", OPTIONAL_TIME_NONE),
                f!("status-autorefresh", "Status Autorefresh", FieldKind::Time),
                f!("shared-users", "Shared Users", OPTIONAL_SHARED_USERS),
                f!("rate-limit", "Rate Limit", FieldKind::Text),
                f!("address-pool", "Address Pool", LOOKUP_POOLS),
                f!("address-list", "Address List", LOOKUP_ADDRESS_LIST),
                f!("incoming-filter", "Incoming Filter", FieldKind::Text),
                f!("outgoing-filter", "Outgoing Filter", FieldKind::Text),
                f!(
                    "incoming-packet-mark",
                    "Incoming Packet Mark",
                    FieldKind::Text
                ),
                f!(
                    "outgoing-packet-mark",
                    "Outgoing Packet Mark",
                    FieldKind::Text
                ),
                f!("add-mac-cookie", "Add MAC Cookie", FieldKind::Toggle),
                f!("mac-cookie-timeout", "MAC Cookie Timeout", FieldKind::Time),
                f!(
                    "open-status-page",
                    "Open Status Page",
                    FieldKind::Enum {
                        values: OPEN_STATUS_PAGE,
                    }
                ),
                f!("transparent-proxy", "Transparent Proxy", FieldKind::Toggle),
                f!("on-login", "On Login", LOOKUP_SCRIPT),
                f!("on-logout", "On Logout", LOOKUP_SCRIPT),
            ],
        },
        FormSection {
            id: "advertise",
            label: "Advertisement",
            read_only: false,
            fields: &[
                f!("advertise", "Advertise", FieldKind::Toggle),
                f!("advertise-url", "Advertise URL", FieldKind::Repeat),
                f!(
                    "advertise-interval",
                    "Advertise Interval",
                    FieldKind::Repeat
                ),
                f!("advertise-timeout", "Advertise Timeout", FieldKind::Time),
            ],
        },
    ],
    create_sections: &[],
};

pub static HOTSPOT_HOST_FORM: FormSchema = FormSchema {
    title_key: "mac-address",
    subtitle_keys: &["address", "server"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                MAC,
                ADDRESS,
                f!("to-address", "To address", FieldKind::Ip),
                f!("server", "Server", LOOKUP_SERVER),
                COMMENT,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("authorized", "Authorized", FieldKind::Readonly),
                f!("bypassed", "Bypassed", FieldKind::Readonly),
                f!("uptime", "Uptime", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub static HOTSPOT_IP_BINDING_FORM: FormSchema = FormSchema {
    title_key: "mac-address",
    subtitle_keys: &["address", "type"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            MAC,
            ADDRESS,
            f!("to-address", "To address", FieldKind::Ip),
            f!("server", "Server", LOOKUP_SERVER),
            f!("type", "Type", KIND_HOTSPOT_BINDING_TYPE),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static HOTSPOT_WALLED_GARDEN_FORM: FormSchema = FormSchema {
    title_key: "dst-host",
    subtitle_keys: &["action"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("dst-host", "Dst host", FieldKind::Text),
            f!("dst-port", "Dst port", FieldKind::Text),
            FIELD_ALLOW_DENY,
            f!("server", "Server", LOOKUP_SERVER),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static HOTSPOT_WALLED_GARDEN_IP_FORM: FormSchema = FormSchema {
    title_key: "dst-address",
    subtitle_keys: &["action"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("dst-address", "Dst address", FieldKind::Ip),
            FIELD_ALLOW_DENY,
            f!("server", "Server", LOOKUP_SERVER),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static PROXY_FORM: FormSchema = FormSchema {
    title_key: "port",
    subtitle_keys: &["enabled"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("enabled", "Enabled", FieldKind::Toggle),
            f!("port", "Port", FieldKind::Number),
            f!("src-address", "Src address", FieldKind::Ip),
            f!("parent-proxy", "Parent proxy", FieldKind::Ip),
            f!("cache-administrator", "Administrator", FieldKind::Text),
            f!("max-cache-size", "Max cache", FieldKind::Text),
        ],
    }],
    create_sections: &[],
};

pub static PROXY_ACCESS_FORM: FormSchema = FormSchema {
    title_key: "src-address",
    subtitle_keys: &["action"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("src-address", "Src address", FieldKind::Ip),
            f!("dst-address", "Dst address", FieldKind::Ip),
            f!("dst-host", "Dst host", FieldKind::Text),
            FIELD_PROXY_ACCESS_ACTION,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static PROXY_CACHE_FORM: FormSchema = FormSchema {
    title_key: "dst-host",
    subtitle_keys: &["action"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("dst-host", "Dst host", FieldKind::Text),
            f!("method", "Method", KIND_HTTP_METHOD),
            FIELD_ALLOW_DENY,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};

pub static PROXY_DIRECT_FORM: FormSchema = FormSchema {
    title_key: "dst-host",
    subtitle_keys: &["action"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("dst-host", "Dst host", FieldKind::Text),
            f!("dst-address", "Dst address", FieldKind::Ip),
            FIELD_ALLOW_DENY,
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};
