//! Feature-owned operator guides for Files screens.

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

pub(crate) static GUIDES: &[(&str, ScreenGuide)] = &[guide!(
    "files",
    "Router filesystem: backups, scripts, images, and uploaded files.",
    "Save a named backup or load a `.backup` file from the action menu (that replaces the \
         running configuration and reboots). Upload (`u`) and download (`d`) copy UTF-8 file \
         bytes between this workstation and `/file` (scripts, certs, exports). Enter on Local \
         Path browses Linux, macOS, and Windows folders (`~/` and `%USERPROFILE%` expand). \
         Directories cannot be downloaded. Download uses `/file/read` in 32 KiB chunks. \
         Binary packages that are not UTF-8 still need Fetch URL (`f`) or another \
         client. Removing a file here deletes it on the router.",
    "name, type, size, creation-time. Contents are fetched only when downloading."
)];
