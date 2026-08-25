//! Size-safe property-sheet geometry.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SheetGeometry {
    pub width: u16,
    pub height: u16,
    pub viewport_height: usize,
}

pub(super) fn sheet_geometry(
    area_width: u16,
    area_height: u16,
    content_rows: usize,
    chrome_rows: u16,
) -> SheetGeometry {
    let width = area_width.saturating_sub(2).min(92);
    let max_height = area_height.saturating_sub(2);
    let needed = chrome_rows.saturating_add(u16::try_from(content_rows.max(1)).unwrap_or(u16::MAX));
    let height = needed.min(max_height);
    SheetGeometry {
        width,
        height,
        viewport_height: usize::from(height.saturating_sub(chrome_rows)).max(1),
    }
}
