//! Feature-owned form schemas for the Queues navigation group.

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

const LOOKUP_QUEUE_SIMPLE: FieldKind = FieldKind::Lookup {
    resource_id: "queue-simple",
    value_key: "name",
    multiple: false,
};
const LOOKUP_QUEUE_TREE: FieldKind = FieldKind::Lookup {
    resource_id: "queue-tree",
    value_key: "name",
    multiple: false,
};
const LOOKUP_QUEUE_TYPE: FieldKind = FieldKind::Lookup {
    resource_id: "queue-type",
    value_key: "name",
    multiple: false,
};

const NAME: FieldSpec = f!("name", "Name", FieldKind::Text);
const COMMENT: FieldSpec = f!("comment", "Comment", FieldKind::Text);
const ENABLED: FieldSpec = f!("disabled", "Enabled", FieldKind::InvertedToggle);
const MAX_LIMIT: FieldSpec = f!("max-limit", "Max Limit", FieldKind::Text);
const LIMIT_AT: FieldSpec = f!("limit-at", "Limit At", FieldKind::Text);
const PRIORITY: FieldSpec = f!("priority", "Priority", FieldKind::Text);
const BUCKET_SIZE: FieldSpec = f!("bucket-size", "Bucket Size", FieldKind::Text);
const BURST_LIMIT: FieldSpec = f!("burst-limit", "Burst Limit", FieldKind::Text);
const BURST_THRESHOLD: FieldSpec = f!("burst-threshold", "Burst Threshold", FieldKind::Text);
const BURST_TIME: FieldSpec = f!("burst-time", "Burst Time", FieldKind::Text);
const TIME: FieldSpec = f!("time", "Time", FieldKind::Text);
const QUEUE_TYPE: FieldSpec = f!("queue", "Queue", LOOKUP_QUEUE_TYPE);

const QUEUE_KIND: FieldKind = FieldKind::Enum {
    values: &[
        "pfifo", "red", "sfq", "pcq", "none", "mq-pfifo", "fq-codel", "cake",
    ],
};

const STATUS_FIELDS: &[FieldSpec] = &[
    f!("rate", "Rate", FieldKind::Readonly),
    f!("packet-rate", "Packet Rate", FieldKind::Readonly),
    f!("queued-bytes", "Queued Bytes", FieldKind::Readonly),
    f!("queued-packets", "Queued Packets", FieldKind::Readonly),
    f!("dropped", "Dropped", FieldKind::Readonly),
    f!("borrowed", "Borrowed", FieldKind::Readonly),
];

pub static QUEUE_SIMPLE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["target"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("target", "Target", FieldKind::Repeat),
                f!("dst", "Dst.", FieldKind::Text),
                f!("parent", "Parent", LOOKUP_QUEUE_SIMPLE),
                f!("packet-marks", "Packet Marks", FieldKind::Repeat),
                MAX_LIMIT,
                LIMIT_AT,
                PRIORITY,
                f!("queue", "Queue", FieldKind::Text),
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "advanced",
            label: "Advanced",
            read_only: false,
            fields: &[BURST_LIMIT, BURST_THRESHOLD, BURST_TIME, BUCKET_SIZE, TIME],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: STATUS_FIELDS,
        },
    ],
    create_sections: &[],
};

pub static QUEUE_TREE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["parent"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                NAME,
                f!("parent", "Parent", LOOKUP_QUEUE_TREE),
                f!("packet-mark", "Packet Mark", FieldKind::Text),
                MAX_LIMIT,
                LIMIT_AT,
                PRIORITY,
                QUEUE_TYPE,
                BURST_LIMIT,
                BURST_THRESHOLD,
                BURST_TIME,
                BUCKET_SIZE,
                TIME,
                COMMENT,
                ENABLED,
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: STATUS_FIELDS,
        },
    ],
    create_sections: &[],
};

pub static QUEUE_TYPE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["kind"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            NAME,
            f!("kind", "Kind", QUEUE_KIND),
            f!("pfifo-limit", "PFIFO Limit", FieldKind::Number),
            f!("sfq-perturb", "SFQ Perturb", FieldKind::Number),
            f!("pcq-rate", "PCQ Rate", FieldKind::Text),
            f!("fq-codel-limit", "FQ-CoDel Limit", FieldKind::Number),
            COMMENT,
        ],
    }],
    create_sections: &[],
};

pub static QUEUE_INTERFACE_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &["queue"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("interface", "Interface", FieldKind::Readonly),
            QUEUE_TYPE,
        ],
    }],
    create_sections: &[],
};
