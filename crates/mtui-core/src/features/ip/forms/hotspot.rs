//! Form schemas for `/ip/hotspot` and `/ip/proxy`.

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

const LOOKUP_IFACE: FieldKind = FieldKind::Lookup {
    resource_id: "interfaces",
    value_key: "name",
    multiple: false,
};
const LOOKUP_POOL: FieldKind = FieldKind::Lookup {
    resource_id: "pools",
    value_key: "name",
    multiple: false,
};
const LOOKUP_PROFILE: FieldKind = FieldKind::Lookup {
    resource_id: "hotspot-profiles",
    value_key: "name",
    multiple: false,
};
const LOOKUP_SERVER: FieldKind = FieldKind::Lookup {
    resource_id: "hotspot",
    value_key: "name",
    multiple: false,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_IFACE);
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
            f!("address-pool", "Address pool", LOOKUP_POOL),
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
            f!("login-by", "Login by", FieldKind::Text),
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
            f!("profile", "Profile", LOOKUP_PROFILE),
            f!("server", "Server", LOOKUP_SERVER),
            COMMENT,
            ENABLED,
        ],
    }],
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
            f!("type", "Type", FieldKind::Text),
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
            f!("action", "Action", FieldKind::Text),
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
            f!("action", "Action", FieldKind::Text),
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
            f!("parent-proxy", "Parent proxy", FieldKind::Text),
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
            f!("action", "Action", FieldKind::Text),
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
            f!("method", "Method", FieldKind::Text),
            f!("action", "Action", FieldKind::Text),
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
            f!("action", "Action", FieldKind::Text),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[],
};
