//! Name-column tree for Files. Indent only when the list is path-ordered.

use std::cmp::Ordering;
use std::collections::HashMap;

pub(crate) type Row = HashMap<String, String>;

#[must_use]
pub(crate) fn is_container(type_value: &str) -> bool {
    matches!(
        type_value.trim().to_ascii_lowercase().as_str(),
        "directory" | "disk"
    )
}

#[must_use]
pub(crate) fn components(name: &str) -> Vec<&str> {
    name.trim()
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .collect()
}

#[must_use]
pub(crate) fn visible_name(name: &str, type_value: &str, tree: bool) -> String {
    if !tree {
        return name.to_string();
    }
    let parts = components(name);
    let depth = parts.len().saturating_sub(1);
    let base = parts.last().copied().unwrap_or(name.trim());
    let mut label = "  ".repeat(depth);
    label.push_str(base);
    if is_container(type_value) && !base.is_empty() && !base.ends_with('/') {
        label.push('/');
    }
    label
}

#[must_use]
pub(crate) fn cmp_rows(left: &Row, right: &Row) -> Ordering {
    let left_name = left.get("name").map_or("", String::as_str);
    let right_name = right.get("name").map_or("", String::as_str);
    let left_dir = is_container(left.get("type").map_or("", String::as_str));
    let right_dir = is_container(right.get("type").map_or("", String::as_str));
    cmp_paths(left_name, left_dir, right_name, right_dir)
}

fn cmp_paths(left: &str, left_dir: bool, right: &str, right_dir: bool) -> Ordering {
    let left_parts = components(left);
    let right_parts = components(right);
    let shared = left_parts.len().min(right_parts.len());
    for i in 0..shared {
        if left_parts[i] != right_parts[i] {
            let left_dir_here = i + 1 < left_parts.len() || left_dir;
            let right_dir_here = i + 1 < right_parts.len() || right_dir;
            match (left_dir_here, right_dir_here) {
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                _ => return left_parts[i].cmp(right_parts[i]),
            }
        }
    }
    left_parts.len().cmp(&right_parts.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, kind: &str) -> Row {
        let mut map = HashMap::new();
        map.insert("name".into(), name.into());
        map.insert("type".into(), kind.into());
        map
    }

    #[test]
    fn parents_sort_before_children() {
        let flash = row("flash", "disk");
        let nested = row("flash/skins/newskin.json", ".json file");
        assert_eq!(cmp_rows(&flash, &nested), Ordering::Less);
        assert_eq!(cmp_rows(&nested, &flash), Ordering::Greater);
    }

    #[test]
    fn nested_paths_stay_with_their_parent() {
        let flash = row("flash", "disk");
        let nested = row("flash/skins", "directory");
        let file = row("export.rsc", "script");
        assert_eq!(cmp_rows(&flash, &file), Ordering::Less);
        assert_eq!(cmp_rows(&nested, &file), Ordering::Less);
        assert_eq!(cmp_rows(&flash, &nested), Ordering::Less);
    }

    #[test]
    fn directories_sort_before_sibling_files() {
        let dir = row("flash/skins", "directory");
        let file = row("flash/export.rsc", "script");
        assert_eq!(cmp_rows(&dir, &file), Ordering::Less);
    }

    #[test]
    fn tree_label_indents_basename() {
        assert_eq!(visible_name("flash", "disk", true), "flash/");
        assert_eq!(visible_name("flash/skins", "directory", true), "  skins/");
        assert_eq!(
            visible_name("flash/skins/newskin.json", ".json file", true),
            "    newskin.json"
        );
        assert_eq!(
            visible_name("flash/skins/newskin.json", ".json file", false),
            "flash/skins/newskin.json"
        );
    }
}
