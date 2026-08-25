//! Deterministic clamped form and picker navigation.

pub(super) fn moved_index(current: usize, delta: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let current = isize::try_from(current).unwrap_or(0);
    let max = isize::try_from(len.saturating_sub(1)).unwrap_or(0);
    usize::try_from((current + delta).clamp(0, max)).unwrap_or(0)
}
