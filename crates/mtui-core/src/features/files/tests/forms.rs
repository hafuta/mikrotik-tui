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
