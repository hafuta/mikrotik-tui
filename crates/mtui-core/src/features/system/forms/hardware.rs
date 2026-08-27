//! Disks, serial, LEDs, `RouterBOARD`, and related System schemas.

use super::common::{
    ENABLED, LOOKUP_DISK, LOOKUP_FILE, LOOKUP_IFACE, LOOKUP_PORT, LOOKUP_USER, ON_EVENT,
};
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

pub(crate) const DISK_TYPES: &[&str] = &[
    "hardware",
    "raid",
    "partition",
    "tmpfs",
    "ramdisk",
    "file",
    "crypted",
    "sshfs",
    "nfs",
    "smb",
    "nvme-tcp",
    "iscsi",
];
pub(crate) const RAID_TYPES: &[&str] = &["0", "1", "4", "5", "6", "linear", "faulty"];
pub(crate) const RAID_CHUNK_SIZES: &[&str] = &["64K", "128K", "256K", "512K", "1M", "2M", "4M"];
pub(crate) const FORMAT_FILE_SYSTEMS: &[&str] = &[
    "ext4",
    "fat32",
    "exfat",
    "xfs",
    "btrfs",
    "discard",
    "discard-secure",
    "wipe",
];
const DISK_TYPE: FieldSpec = f!("type", "Type", FieldKind::Enum { values: DISK_TYPES });
const DISK_SLOT: FieldSpec = f!("slot", "Slot", FieldKind::Text);

pub static DISK_FORM: FormSchema = FormSchema {
    title_key: "slot",
    subtitle_keys: &["type"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                DISK_SLOT,
                DISK_TYPE,
                f!("parent", "Parent", LOOKUP_DISK),
                f!("mount-filesystem", "Mount Filesystem", FieldKind::Toggle),
                f!("mount-read-only", "Mount Read Only", FieldKind::Toggle),
                f!("swap", "Swap", FieldKind::Toggle),
                ENABLED,
                f!("tmpfs-max-size", "Tmpfs Max Size", FieldKind::Number),
                f!("ramdisk-size", "Ramdisk Size", FieldKind::Number),
                f!("partition-number", "Partition Number", FieldKind::Number),
                f!("partition-offset", "Partition Offset", FieldKind::Number),
                f!("partition-size", "Partition Size", FieldKind::Number),
                f!(
                    "raid-type",
                    "RAID Type",
                    FieldKind::Enum { values: RAID_TYPES }
                ),
                f!("raid-device-count", "RAID Device Count", FieldKind::Number),
                f!(
                    "raid-max-component-size",
                    "RAID Max Component Size",
                    FieldKind::Number
                ),
                f!(
                    "raid-chunk-size",
                    "RAID Chunk Size",
                    FieldKind::Enum {
                        values: RAID_CHUNK_SIZES,
                    }
                ),
                f!("raid-master", "RAID Master", LOOKUP_DISK),
                f!("raid-role", "RAID Role", FieldKind::Number),
                f!(
                    "raid-member-failed",
                    "RAID Member Failed",
                    FieldKind::Toggle
                ),
                f!("file-path", "File Path", LOOKUP_FILE),
                f!("file-size", "File Size", FieldKind::Number),
                f!("file-offset", "File Offset", FieldKind::Number),
                f!("crypted-backend", "Crypted Backend", LOOKUP_DISK),
                f!("encryption-key", "Encryption Key", FieldKind::Secret),
                f!("sshfs-address", "SSHFS Address", FieldKind::Text),
                f!("sshfs-port", "SSHFS Port", FieldKind::Number),
                f!("sshfs-user", "SSHFS User", FieldKind::Text),
                f!("sshfs-password", "SSHFS Password", FieldKind::Secret),
                f!("sshfs-path", "SSHFS Path", FieldKind::Text),
                f!("nfs-address", "NFS Address", FieldKind::Text),
                f!("nfs-share", "NFS Share", FieldKind::Text),
                f!("smb-address", "SMB Address", FieldKind::Text),
                f!("smb-share", "SMB Share", FieldKind::Text),
                f!("smb-user", "SMB User", FieldKind::Text),
                f!("smb-password", "SMB Password", FieldKind::Secret),
                f!("smb-encryption", "SMB Encryption", FieldKind::Toggle),
                f!("nvme-tcp-address", "NVMe TCP Address", FieldKind::Text),
                f!("nvme-tcp-nqn", "NVMe TCP NQN", FieldKind::Text),
                f!("nvme-tcp-host-name", "NVMe TCP Host Name", FieldKind::Text),
                f!("nvme-tcp-password", "NVMe TCP Password", FieldKind::Secret),
                f!("nvme-tcp-port", "NVMe TCP Port", FieldKind::Number),
                f!("iscsi-address", "iSCSI Address", FieldKind::Text),
                f!("iscsi-iqn", "iSCSI IQN", FieldKind::Text),
                f!("iscsi-port", "iSCSI Port", FieldKind::Number),
                f!("nvme-tcp-export", "NVMe TCP Export", FieldKind::Toggle),
                f!(
                    "nvme-tcp-server-port",
                    "NVMe TCP Server Port",
                    FieldKind::Number
                ),
                f!(
                    "nvme-tcp-server-nqn",
                    "NVMe TCP Server NQN",
                    FieldKind::Text
                ),
                f!(
                    "nvme-tcp-server-password",
                    "NVMe TCP Server Password",
                    FieldKind::Secret
                ),
                f!("iscsi-export", "iSCSI Export", FieldKind::Toggle),
                f!("iscsi-server-port", "iSCSI Server Port", FieldKind::Number),
                f!("iscsi-server-iqn", "iSCSI Server IQN", FieldKind::Text),
                f!("nfs-sharing", "NFS Sharing", FieldKind::Toggle),
                f!("smb-sharing", "SMB Sharing", FieldKind::Toggle),
                f!("smb-server-user", "SMB Server User", LOOKUP_USER),
                f!(
                    "smb-server-password",
                    "SMB Server Password",
                    FieldKind::Secret
                ),
                f!(
                    "smb-server-encryption",
                    "SMB Server Encryption",
                    FieldKind::Toggle
                ),
                f!("media-sharing", "Media Sharing", FieldKind::Toggle),
                f!("media-interface", "Media Interface", LOOKUP_IFACE),
                f!(
                    "self-encryption-password",
                    "Self Encryption Password",
                    FieldKind::Secret
                ),
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("model", "Model", FieldKind::Readonly),
                f!("serial", "Serial", FieldKind::Readonly),
                f!("fw-version", "FW Version", FieldKind::Readonly),
                f!("size", "Size", FieldKind::Readonly),
                f!("free", "Free", FieldKind::Readonly),
                f!("fs", "FS", FieldKind::Readonly),
                f!("fs-label", "FS Label", FieldKind::Readonly),
                f!("fs-uuid", "FS UUID", FieldKind::Readonly),
                f!("state", "State", FieldKind::Readonly),
                f!("mount-point", "Mount Point", FieldKind::Readonly),
                f!("slot-default", "Slot Default", FieldKind::Readonly),
                f!("raid-uuid", "RAID UUID", FieldKind::Readonly),
                f!(
                    "raid-member-state",
                    "RAID Member State",
                    FieldKind::Readonly
                ),
            ],
        },
    ],
    create_sections: &[],
};

const FORMAT_DISK_FIELDS: &[FieldSpec] = &[
    FieldSpec {
        key: "file-system",
        label: "File System",
        kind: FieldKind::Enum {
            values: FORMAT_FILE_SYSTEMS,
        },
    },
    f!("label", "Label", FieldKind::Text),
    f!(
        "mbr-partition-table",
        "MBR Partition Table",
        FieldKind::Toggle
    ),
];

pub static FORMAT_DISK_PROMPT: FormSchema = FormSchema {
    title_key: "file-system",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: FORMAT_DISK_FIELDS,
    }],
};

const CONSOLE_GENERAL: &[FieldSpec] = &[
    f!("port", "Port", LOOKUP_PORT),
    f!("term", "Term", FieldKind::Text),
    f!("channel", "Channel", FieldKind::Number),
    ENABLED,
];

pub static CONSOLE_FORM: FormSchema = FormSchema {
    title_key: "port",
    subtitle_keys: &["term"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: CONSOLE_GENERAL,
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("vcno", "VCNO", FieldKind::Readonly),
                f!("used", "Used", FieldKind::Readonly),
                f!("free", "Free", FieldKind::Readonly),
                f!("wedged", "Wedged", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub(crate) const LED_TYPES: &[&str] = &[
    "off",
    "on",
    "modem-status",
    "interface-status",
    "interface-activity",
    "wireless-status",
    "wireless-signal-strength",
    "poe-out",
    "flash-access",
    "rb-capsman",
    "rb-wps",
    "fan-fault",
    "gps-fix",
    "ap-cap",
];

const LED_GENERAL: &[FieldSpec] = &[
    FieldSpec {
        key: "type",
        label: "Type",
        kind: FieldKind::Enum { values: LED_TYPES },
    },
    f!("interface", "Interface", LOOKUP_IFACE),
    f!("modem", "Modem", LOOKUP_IFACE),
    f!("leds", "LEDs", FieldKind::Repeat),
    ENABLED,
];

pub static LED_FORM: FormSchema = FormSchema {
    title_key: "type",
    subtitle_keys: &["interface"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: LED_GENERAL,
    }],
    create_sections: &[],
};

pub(crate) const LED_ALL_OFF: &[&str] = &["never", "immediately", "after-1h"];

pub static LED_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "all-leds-off",
    subtitle_keys: &[],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[FieldSpec {
            key: "all-leds-off",
            label: "All LEDs Off",
            kind: FieldKind::Enum {
                values: LED_ALL_OFF,
            },
        }],
    }],
    create_sections: &[],
};

pub(crate) const PORT_BAUD: &[&str] = &[
    "auto", "110", "300", "600", "1200", "2400", "4800", "9600", "19200", "38400", "57600",
    "115200", "230400", "460800", "921600",
];
pub(crate) const PORT_DATA_BITS: &[&str] = &["7", "8"];
pub(crate) const PORT_PARITY: &[&str] = &["none", "even", "odd"];
pub(crate) const PORT_STOP_BITS: &[&str] = &["1", "2"];
pub(crate) const PORT_FLOW: &[&str] = &["none", "hardware", "xon-xoff"];

pub static PORT_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &["baud-rate"],
    sections: &[
        FormSection {
            id: "general",
            label: "General",
            read_only: false,
            fields: &[
                f!("name", "Name", FieldKind::Readonly),
                FieldSpec {
                    key: "baud-rate",
                    label: "Baud Rate",
                    kind: FieldKind::Enum { values: PORT_BAUD },
                },
                FieldSpec {
                    key: "data-bits",
                    label: "Data Bits",
                    kind: FieldKind::Enum {
                        values: PORT_DATA_BITS,
                    },
                },
                FieldSpec {
                    key: "parity",
                    label: "Parity",
                    kind: FieldKind::Enum {
                        values: PORT_PARITY,
                    },
                },
                FieldSpec {
                    key: "stop-bits",
                    label: "Stop Bits",
                    kind: FieldKind::Enum {
                        values: PORT_STOP_BITS,
                    },
                },
                FieldSpec {
                    key: "flow-control",
                    label: "Flow Control",
                    kind: FieldKind::Enum { values: PORT_FLOW },
                },
            ],
        },
        FormSection {
            id: "status",
            label: "Status",
            read_only: true,
            fields: &[
                f!("used", "Used", FieldKind::Readonly),
                f!("free", "Free", FieldKind::Readonly),
            ],
        },
    ],
    create_sections: &[],
};

pub(crate) const BOOT_OS: &[&str] = &["router-os", "swos"];
pub(crate) const BOOT_DEVICE: &[&str] = &[
    "nand-if-fail-then-ethernet",
    "nand-only",
    "ethernet",
    "try-ethernet-once-then-nand",
    "flash-boot",
    "flash-boot-once-then-nand",
];
pub(crate) const BOOT_PROTOCOL: &[&str] = &["bootp", "dhcp"];
pub(crate) const PROTECTED_ROUTERBOOT: &[&str] = &["disabled", "enabled"];
/// Board-specific `cpu-frequency` values; the picker also keeps the printed value.
pub(crate) const CPU_FREQUENCY: &[&str] = &[
    "auto", "400MHz", "600MHz", "650MHz", "716MHz", "800MHz", "880MHz", "1000MHz", "1200MHz",
    "1400MHz", "1500MHz", "1800MHz", "2000MHz",
];
/// Board-specific `memory-frequency` values; the picker also keeps the printed value.
pub(crate) const MEMORY_FREQUENCY: &[&str] = &["auto", "800DDR", "1066DDR", "1200DDR", "1333DDR"];

pub static ROUTERBOARD_SETTINGS_FORM: FormSchema = FormSchema {
    title_key: "boot-device",
    subtitle_keys: &["boot-os"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[
            f!("auto-upgrade", "Auto Upgrade", FieldKind::Toggle),
            FieldSpec {
                key: "boot-device",
                label: "Boot Device",
                kind: FieldKind::Enum {
                    values: BOOT_DEVICE,
                },
            },
            FieldSpec {
                key: "boot-os",
                label: "Boot OS",
                kind: FieldKind::Enum { values: BOOT_OS },
            },
            FieldSpec {
                key: "boot-protocol",
                label: "Boot Protocol",
                kind: FieldKind::Enum {
                    values: BOOT_PROTOCOL,
                },
            },
            FieldSpec {
                key: "cpu-frequency",
                label: "CPU Frequency",
                kind: FieldKind::Enum {
                    values: CPU_FREQUENCY,
                },
            },
            FieldSpec {
                key: "memory-frequency",
                label: "Memory Frequency",
                kind: FieldKind::Enum {
                    values: MEMORY_FREQUENCY,
                },
            },
            f!(
                "enable-jumper-reset",
                "Enable Jumper Reset",
                FieldKind::Toggle
            ),
            f!(
                "force-backup-booter",
                "Force Backup Booter",
                FieldKind::Toggle
            ),
            f!("silent-boot", "Silent Boot", FieldKind::Toggle),
            FieldSpec {
                key: "protected-routerboot",
                label: "Protected RouterBOOT",
                kind: FieldKind::Enum {
                    values: PROTECTED_ROUTERBOOT,
                },
            },
        ],
    }],
    create_sections: &[],
};

const BUTTON_GENERAL: &[FieldSpec] = &[
    f!("enabled", "Enabled", FieldKind::Toggle),
    f!("hold-time", "Hold Time", FieldKind::Time),
    ON_EVENT,
];

pub static ROUTERBOARD_MODE_BUTTON_FORM: FormSchema = FormSchema {
    title_key: "on-event",
    subtitle_keys: &["hold-time"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: BUTTON_GENERAL,
    }],
    create_sections: &[],
};

pub static ROUTERBOARD_RESET_BUTTON_FORM: FormSchema = FormSchema {
    title_key: "on-event",
    subtitle_keys: &["hold-time"],
    sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: BUTTON_GENERAL,
    }],
    create_sections: &[],
};

pub static USB_POWER_RESET_PROMPT: FormSchema = FormSchema {
    title_key: "duration",
    subtitle_keys: &[],
    sections: &[],
    create_sections: &[FormSection {
        id: "general",
        label: "General",
        read_only: false,
        fields: &[f!("duration", "Duration", FieldKind::Time)],
    }],
};
