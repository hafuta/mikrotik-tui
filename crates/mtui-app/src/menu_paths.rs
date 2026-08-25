//! Walk `/console/inspect` for catalog paths so missing menus stay hidden.

use std::collections::{HashMap, HashSet};

use mtui_core::{ALL_RESOURCES, menu_path_segments, unavailable_from_menu_tree};
use mtui_routeros::{Client, Result};

/// Resource ids whose `RouterOS` command path is not on this device.
pub async fn probe_missing_resource_ids(client: &Client) -> Result<HashSet<String>> {
    let tree = inspect_catalog_tree(client).await?;
    Ok(unavailable_from_menu_tree(&tree).into_keys().collect())
}

async fn inspect_catalog_tree(client: &Client) -> Result<HashMap<String, HashSet<String>>> {
    let mut tree = HashMap::new();
    tree.insert(String::new(), client.inspect_children(&[]).await?);

    for spec in ALL_RESOURCES.iter() {
        let Some(segments) = menu_path_segments(spec) else {
            continue;
        };
        let mut prefix = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            let parent_key = prefix.join(",");
            if !tree
                .get(&parent_key)
                .is_some_and(|children| children.contains(*segment))
            {
                break;
            }
            prefix.push(*segment);
            if index + 1 >= segments.len() {
                continue;
            }
            let key = prefix.join(",");
            if tree.contains_key(&key) {
                continue;
            }
            let children = client.inspect_children(&prefix).await?;
            tree.insert(key, children);
        }
    }
    Ok(tree)
}
