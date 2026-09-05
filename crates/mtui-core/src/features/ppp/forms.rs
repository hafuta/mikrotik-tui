//! Feature-owned form schemas for the complete PPP navigation group.

use crate::form_fields::{
    KIND_USE_IPSEC_REQUIRE, LOOKUP_INTERFACE_LISTS, LOOKUP_INTERFACES, LOOKUP_POOLS, LOOKUP_PORTS,
};
use crate::forms::{EnumChoice, FieldKind, FieldSpec, FormSchema, FormSection};

macro_rules! f {
    ($key:literal, $label:literal, $kind:expr) => {
        FieldSpec {
            key: $key,
            label: $label,
            kind: $kind,
        }
    };
}

const LOOKUP_PPP_PROFILE: FieldKind = FieldKind::Lookup {
    resource_id: "ppp-profiles",
    value_key: "name",
    multiple: false,
};
const LOOKUP_BRIDGE: FieldKind = FieldKind::Lookup {
    resource_id: "bridges",
    value_key: "name",
    multiple: false,
};
const LOOKUP_CERT: FieldKind = FieldKind::Lookup {
    resource_id: "certificates",
    value_key: "name",
    multiple: false,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const USER: FieldSpec = f!("user", "User", FieldKind::Text);
const PASSWORD: FieldSpec = f!("password", "Password", FieldKind::Secret);
const PROFILE: FieldSpec = f!("profile", "Profile", LOOKUP_PPP_PROFILE);
const RUNNING: FieldSpec = f!("running", "Running", FieldKind::Readonly);
const INTERFACE: FieldSpec = f!("interface", "Interface", LOOKUP_INTERFACES);
const PORT: FieldSpec = f!("port", "Port", LOOKUP_PORTS);
const CONNECT_TO: FieldSpec = f!("connect-to", "Connect To", FieldKind::Text);
const ADD_DEFAULT_ROUTE: FieldSpec =
    f!("add-default-route", "Add Default Route", FieldKind::Toggle);
const DEFAULT_PROFILE: FieldSpec = f!("default-profile", "Default Profile", LOOKUP_PPP_PROFILE);
const AUTHENTICATION: FieldSpec = f!("authentication", "Authentication", FieldKind::Repeat);
const KEEPALIVE: FieldSpec = f!("keepalive-timeout", "Keepalive Timeout", FieldKind::Time);
const MAX_MTU: FieldSpec = f!("max-mtu", "Max MTU", FieldKind::Number);
const MAX_MRU: FieldSpec = f!("max-mru", "Max MRU", FieldKind::Number);
const SERVER_ENABLED: FieldSpec = f!("enabled", "Enabled", FieldKind::Toggle);
const CERTIFICATE: FieldSpec = f!("certificate", "Certificate", LOOKUP_CERT);
const USE_IPSEC: FieldSpec = f!("use-ipsec", "Use IPsec", FieldKind::Toggle);
const IPSEC_SECRET: FieldSpec = f!("ipsec-secret", "IPsec Secret", FieldKind::Secret);
const SERVICE_NAME: FieldSpec = f!("service-name", "Service Name", FieldKind::Text);
const PPP_SERVICE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "any",
        value: "any",
    },
    EnumChoice {
        label: "async",
        value: "async",
    },
    EnumChoice {
        label: "l2tp",
        value: "l2tp",
    },
    EnumChoice {
        label: "ovpn",
        value: "ovpn",
    },
    EnumChoice {
        label: "pppoe",
        value: "pppoe",
    },
    EnumChoice {
        label: "pptp",
        value: "pptp",
    },
    EnumChoice {
        label: "sstp",
        value: "sstp",
    },
];
const YES_NO_DEFAULT: &[EnumChoice] = &[
    EnumChoice {
        label: "default",
        value: "default",
    },
    EnumChoice {
        label: "yes",
        value: "yes",
    },
    EnumChoice {
        label: "no",
        value: "no",
    },
];
const OVPN_MODE_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "ip",
        value: "ip",
    },
    EnumChoice {
        label: "ethernet",
        value: "ethernet",
    },
];
const OVPN_AUTH_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "sha1",
        value: "sha1",
    },
    EnumChoice {
        label: "md5",
        value: "md5",
    },
    EnumChoice {
        label: "sha256",
        value: "sha256",
    },
    EnumChoice {
        label: "sha512",
        value: "sha512",
    },
    EnumChoice {
        label: "null",
        value: "null",
    },
];
const OVPN_CIPHER_CHOICES: &[EnumChoice] = &[
    EnumChoice {
        label: "aes128-cbc",
        value: "aes128-cbc",
    },
    EnumChoice {
        label: "aes192-cbc",
        value: "aes192-cbc",
    },
    EnumChoice {
        label: "aes256-cbc",
        value: "aes256-cbc",
    },
    EnumChoice {
        label: "aes128-gcm",
        value: "aes128-gcm",
    },
    EnumChoice {
        label: "aes192-gcm",
        value: "aes192-gcm",
    },
    EnumChoice {
        label: "aes256-gcm",
        value: "aes256-gcm",
    },
    EnumChoice {
        label: "blowfish128",
        value: "blowfish128",
    },
    EnumChoice {
        label: "null",
        value: "null",
    },
];
const SERVICE: FieldSpec = f!(
    "service",
    "Service",
    FieldKind::LabeledEnum {
        choices: PPP_SERVICE_CHOICES
    }
);
const MODE: FieldSpec = f!(
    "mode",
    "Mode",
    FieldKind::LabeledEnum {
        choices: OVPN_MODE_CHOICES
    }
);
const AUTH: FieldSpec = f!(
    "auth",
    "Authentication",
    FieldKind::LabeledEnum {
        choices: OVPN_AUTH_CHOICES
    }
);
const CIPHER: FieldSpec = f!(
    "cipher",
    "Cipher",
    FieldKind::LabeledEnum {
        choices: OVPN_CIPHER_CHOICES
    }
);

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
            SERVICE,
            PROFILE,
            f!("caller-id", "Caller ID", FieldKind::Text),
            f!("local-address", "Local Address", FieldKind::Ip),
            f!("remote-address", "Remote Address", FieldKind::Ip),
            f!("remote-ipv6-prefix", "Remote IPv6 Prefix", FieldKind::Ipv6),
            COMMENT,
            ENABLED,
        ],
    }],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[NAME, PASSWORD, SERVICE, PROFILE, COMMENT],
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
            f!("local-address", "Local Address", LOOKUP_POOLS),
            f!("remote-address", "Remote Address", LOOKUP_POOLS),
            f!("dns-server", "DNS Server", FieldKind::Repeat),
            f!("rate-limit", "Rate Limit", FieldKind::Text),
            f!(
                "only-one",
                "Only One",
                FieldKind::LabeledEnum {
                    choices: YES_NO_DEFAULT
                }
            ),
            f!(
                "use-encryption",
                "Use Encryption",
                FieldKind::LabeledEnum {
                    choices: YES_NO_DEFAULT
                }
            ),
            f!(
                "use-compression",
                "Use Compression",
                FieldKind::LabeledEnum {
                    choices: YES_NO_DEFAULT
                }
            ),
            f!(
                "change-tcp-mss",
                "Change TCP MSS",
                FieldKind::LabeledEnum {
                    choices: YES_NO_DEFAULT
                }
            ),
            f!("bridge", "Bridge", LOOKUP_BRIDGE),
            f!("interface-list", "Interface List", LOOKUP_INTERFACE_LISTS),
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
            f!("use-radius", "Use RADIUS", FieldKind::Toggle),
            f!("accounting", "Accounting", FieldKind::Toggle),
            f!("interim-update", "Interim Update", FieldKind::Time),
            f!(
                "enable-ipv6-accounting",
                "Enable IPv6 Accounting",
                FieldKind::Toggle
            ),
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
                PORT,
                USER,
                PASSWORD,
                PROFILE,
                f!("phone", "Phone", FieldKind::Text),
                ADD_DEFAULT_ROUTE,
                COMMENT,
                ENABLED,
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
        fields: &[NAME, PORT, USER, PASSWORD, COMMENT],
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
                f!("ac-name", "AC Name", FieldKind::Text),
                PROFILE,
                ADD_DEFAULT_ROUTE,
                f!("use-peer-dns", "Use Peer DNS", FieldKind::Toggle),
                COMMENT,
                ENABLED,
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
            f!(
                "one-session-per-host",
                "One Session Per Host",
                FieldKind::Toggle
            ),
            f!("max-sessions", "Max Sessions", FieldKind::Number),
            KEEPALIVE,
            COMMENT,
            ENABLED,
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
                ENABLED,
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
            SERVER_ENABLED,
            DEFAULT_PROFILE,
            AUTHENTICATION,
            KEEPALIVE,
            MAX_MTU,
            MAX_MRU,
            f!("mrru", "MRRU", FieldKind::Number),
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
                ENABLED,
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
            SERVER_ENABLED,
            DEFAULT_PROFILE,
            AUTHENTICATION,
            f!("use-ipsec", "Use IPsec", KIND_USE_IPSEC_REQUIRE),
            IPSEC_SECRET,
            KEEPALIVE,
            MAX_MTU,
            MAX_MRU,
            f!("allow-fast-path", "Allow Fast Path", FieldKind::Toggle),
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
                f!(
                    "verify-server-certificate",
                    "Verify Server Certificate",
                    FieldKind::Toggle
                ),
                ADD_DEFAULT_ROUTE,
                COMMENT,
                ENABLED,
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
            SERVER_ENABLED,
            CERTIFICATE,
            DEFAULT_PROFILE,
            AUTHENTICATION,
            f!("port", "Port", FieldKind::Number),
            f!(
                "verify-client-certificate",
                "Verify Client Certificate",
                FieldKind::Toggle
            ),
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
                ENABLED,
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
            SERVER_ENABLED,
            f!("port", "Port", FieldKind::Number),
            MODE,
            f!(
                "netmask",
                "Netmask",
                FieldKind::ConstrainedNumber {
                    min: Some(0),
                    max: Some(32),
                }
            ),
            CERTIFICATE,
            DEFAULT_PROFILE,
            f!("auth", "Authentication", FieldKind::Repeat),
            f!("cipher", "Cipher", FieldKind::Repeat),
            f!(
                "require-client-certificate",
                "Require Client Certificate",
                FieldKind::Toggle
            ),
        ],
    }],
    create_sections: &[],
};
