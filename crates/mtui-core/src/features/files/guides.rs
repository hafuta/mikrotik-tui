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
         running configuration and reboots). Pull a file onto the router with /tool/fetch (`f`). \
         Removing a file here deletes it on the router. Local contents upload/download is not \
         available over the classic API.",
    "name, type, size, creation-time. Contents are not shown in the table."
)];
