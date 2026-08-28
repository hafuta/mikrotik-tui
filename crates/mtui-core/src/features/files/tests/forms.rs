use std::collections::HashMap;

use crate::features::files::guides::GUIDES;
use crate::features::files::resources::RESOURCES;
use crate::features::files::rules::form_field_state;

#[test]
fn catalog_and_guides_cover_the_files_group() {
    assert_eq!(RESOURCES.len(), 1);
    assert_eq!(GUIDES.len(), 1);
    for spec in RESOURCES {
        assert!(
            GUIDES.iter().any(|(id, _)| *id == spec.id),
            "missing guide for {}",
            spec.id
        );
        assert!(spec.form.is_none());
    }
    assert!(form_field_state("files", "name", &HashMap::new()).is_none());
}

#[test]
fn upload_and_download_prompts_use_general_sections() {
    use crate::forms::FieldKind;
    use crate::{DOWNLOAD_FORM, UPLOAD_FORM};

    let upload = UPLOAD_FORM.sections_for(true);
    assert_eq!(upload.len(), 1);
    assert_eq!(upload[0].id, "general");
    assert_eq!(
        upload[0]
            .fields
            .iter()
            .map(|field| field.key)
            .collect::<Vec<_>>(),
        ["local-path", "remote-name"]
    );
    assert!(matches!(upload[0].fields[0].kind, FieldKind::Text));
    assert_eq!(upload[0].fields[0].label, "Local Path");
    assert_eq!(upload[0].fields[1].label, "Remote Name");

    let download = DOWNLOAD_FORM.sections_for(true);
    assert_eq!(download.len(), 1);
    assert_eq!(download[0].id, "general");
    assert_eq!(download[0].fields[0].key, "local-path");
    assert_eq!(download[0].fields[0].label, "Local Path");
    assert!(DOWNLOAD_FORM.create_sections.is_empty());
    assert!(UPLOAD_FORM.create_sections.is_empty());
}
