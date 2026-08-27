//! Feature-owned operator guides for `Container` screens.

use crate::about::ScreenGuide;

macro_rules! guide {
    ($id:literal, $summary:literal, $when:literal, $fields:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: None,
            },
        )
    };
    ($id:literal, $summary:literal, $when:literal, $fields:literal, $docs:literal) => {
        (
            $id,
            ScreenGuide {
                summary: $summary,
                use_when: $when,
                fields: $fields,
                docs_url: Some($docs),
            },
        )
    };
}

pub(crate) static GUIDES: &[(&str, ScreenGuide)] = &[
    guide!(
        "containers",
        "Linux containers on RouterOS v7. Images come from a registry (remote-image) or a \
         tar already on Files. Adding a row starts download or extract; it does not start \
         the container. Status, arch, OS, and tag are what the device stored after extract.",
        "Needs the container extra package (arm, arm64, x86, CHR). Device-mode container=yes \
         needs a reset or mode button, or a cold power-off on x86, within the timeout. DNS \
         must be set on IP DNS or on the container. EN7562CT boards only run arm32v5 images; \
         the registry rejects other architectures. This client does not filter image names.",
        "name, interface (VETH), remote-image or file, root-dir, envlist, mountlists, \
         start-on-boot, logging, memory limits, healthcheck (7.23+), status/arch/tag.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-config",
        "Global container settings: registry URL, extract directory, layer store, and \
         registry username/password.",
        "Set registry-url and tmpdir on disk before a remote-image add. Password is stored \
         on the router; this client masks it in the sheet.",
        "registry-url, tmpdir, layer-dir, username, password, memory-high/max, swap-max, \
         assumed-registry-url, memory-current.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-envs",
        "Named environment lists. Each row is a list name plus one key and value. A \
         container points at a list with envlist.",
        "Group variables per app. RouterOS does not mark env values as secrets.",
        "list, key, value.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "container-mounts",
        "Named bind-mount lists. Each row is list, host src, and path inside the container. \
         Containers reference lists with mountlists.",
        "Point src at a disk path that already exists on the router.",
        "list, src, dst.",
        "https://manual.mikrotik.com/docs/containers/"
    ),
    guide!(
        "apps",
        "MikroTik app catalog on top of containers. YAML plus NAT and veth that RouterOS \
         applies. arm64 and x86 only; EN7562CT is not supported for Apps even when \
         containers are.",
        "Use it when you want a packaged app instead of a raw container row. The device \
         fetches the catalog; this client lists /app.",
        "name, network (internal/lan/default), YAML, environment/mounts/redirects, status, \
         UI URL, IP.",
        "https://manual.mikrotik.com/docs/containers/apps/"
    ),
];
