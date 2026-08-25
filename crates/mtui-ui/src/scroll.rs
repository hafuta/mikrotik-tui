//! Viewport math and chrome for internally scrolled overlay lists.

/// Visible window of a list that follows `focus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollView {
    pub offset: usize,
    pub visible: usize,
    pub total: usize,
}

impl ScrollView {
    /// Keep `focus` inside a `visible`-row window. The focused row sits on the
    /// last line when moving down, and the first line when it would leave the top.
    #[must_use]
    pub fn around_focus(focus: usize, visible: usize, total: usize) -> Self {
        let visible = visible.max(1);
        if total == 0 {
            return Self {
                offset: 0,
                visible,
                total: 0,
            };
        }
        let visible = visible.min(total);
        let max_off = total.saturating_sub(visible);
        let offset = focus.saturating_sub(visible.saturating_sub(1)).min(max_off);
        Self {
            offset,
            visible,
            total,
        }
    }

    #[must_use]
    pub fn overflows(self) -> bool {
        self.total > self.visible
    }

    #[must_use]
    pub fn end(self) -> usize {
        self.total.min(self.offset.saturating_add(self.visible))
    }

    /// `6-12/17` when the list overflows; empty otherwise.
    #[must_use]
    pub fn range_label(self) -> String {
        if !self.overflows() || self.total == 0 {
            return String::new();
        }
        format!(
            "{}-{}/{}",
            self.offset.saturating_add(1),
            self.end(),
            self.total
        )
    }

    /// Right-edge track (`│`) with a thumb (`▐`) mapped onto the window.
    #[must_use]
    pub fn gutter(self, row_in_window: usize) -> char {
        if !self.overflows() {
            return ' ';
        }
        let thumb_h = (self.visible.saturating_mul(self.visible) / self.total.max(1))
            .max(1)
            .min(self.visible);
        let travel = self.visible.saturating_sub(thumb_h);
        let max_off = self.total.saturating_sub(self.visible);
        let thumb_start = self
            .offset
            .saturating_mul(travel)
            .checked_div(max_off)
            .unwrap_or(0);
        if row_in_window >= thumb_start && row_in_window < thumb_start.saturating_add(thumb_h) {
            '▐'
        } else {
            '│'
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn around_focus_pins_the_last_row_when_moving_down() {
        let view = ScrollView::around_focus(16, 8, 17);
        assert_eq!(view.offset, 9);
        assert_eq!(view.end(), 17);
        assert_eq!(view.range_label(), "10-17/17");
        assert_eq!(view.gutter(0), '│');
        assert_eq!(view.gutter(7), '▐');
    }

    #[test]
    fn short_lists_need_no_chrome() {
        let view = ScrollView::around_focus(1, 8, 3);
        assert!(!view.overflows());
        assert!(view.range_label().is_empty());
        assert_eq!(view.gutter(0), ' ');
    }
}
