//! Form schemas for `/ip/ipsec` (`IPsec`) menus.

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
const ADDRESS: FieldSpec = f!("address", "Address", FieldKind::Ip);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
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
                ENABLED,
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
    create_sections: &[],
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
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[DYNAMIC],
        },
    ],
    create_sections: &[],
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
                f!("action", "Action", FieldKind::Text),
                f!("level", "Level", FieldKind::Text),
                f!("ipsec-protocols", "IPsec protocols", FieldKind::Text),
                PROPOSAL,
                PEER,
                f!("tunnel", "Tunnel", FieldKind::Toggle),
                f!("sa-src-address", "SA src", FieldKind::Ip),
                f!("sa-dst-address", "SA dst", FieldKind::Ip),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "match",
            label: "Match",
            read_only: false,
            fields: &[
                f!("src-address", "Src address", FieldKind::Ip),
                f!("dst-address", "Dst address", FieldKind::Ip),
                f!("src-port", "Src port", FieldKind::Text),
                f!("dst-port", "Dst port", FieldKind::Text),
                f!("protocol", "Protocol", FieldKind::Text),
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
    create_sections: &[],
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
            ENABLED,
        ],
    }],
    create_sections: &[],
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
    create_sections: &[],
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
            ENABLED,
        ],
    }],
    create_sections: &[],
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
    create_sections: &[],
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
    create_sections: &[],
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
