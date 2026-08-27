//! Property-sheet row and scalar-control rendering.

use mtui_core::{FieldKind, FieldSpec, ScalarKind};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::layout::clip_line;
use crate::styles::Styles;

use super::{FormRow, FormSession, enum_display_value};

const LABEL_COLS: usize = 22;
const TAG_COLS: usize = 6;
/// Same eight-bullet token `RouterOS` rows use in tables (`MASKED_VALUE`).
const SECRET_PLACEHOLDER: &str = "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}";

/// One bullet per input character so typing has a visible length.
#[must_use]
pub(super) fn secret_input_mask(raw: &str) -> String {
    if raw.is_empty() {
        String::new()
    } else {
        "\u{2022}".repeat(raw.chars().count())
    }
}

#[must_use]
pub(super) fn is_secret_placeholder(raw: &str) -> bool {
    raw == SECRET_PLACEHOLDER
}

pub(super) fn row_line(
    session: &FormSession,
    row: FormRow<'_>,
    focused: bool,
    width: usize,
    gutter: char,
    styles: &Styles,
) -> Line<'static> {
    let field = row.field();
    let locked = row.locked();
    let caret = if focused { ">" } else { " " };
    let (label, tag) = match row {
        FormRow::RepeatItem { index, .. } if index > 0 => ("", ""),
        FormRow::RepeatAdd { .. } => ("", "list"),
        _ => (field.label, field.kind.tag()),
    };
    let label = pad_visual(label, LABEL_COLS);
    let tag = pad_visual(tag, TAG_COLS);
    let label_style = if focused { styles.focus } else { styles.muted };
    let tag_style = if focused { styles.key } else { styles.quiet };
    let mut spans = vec![
        Span::styled(format!("{caret} {label} "), label_style),
        Span::styled(format!("{tag} "), tag_style),
    ];
    let used = spans
        .iter()
        .map(|span| span.content.as_ref().width())
        .sum::<usize>();
    let gutter_w = usize::from(gutter != ' ');
    let rest = width.saturating_sub(used).saturating_sub(gutter_w);
    let raw = match row {
        FormRow::RepeatItem { index, .. } => session
            .repeat
            .get(field.key)
            .and_then(|items| items.get(index))
            .map_or("", String::as_str),
        FormRow::RepeatAdd { .. } => "",
        FormRow::Field { .. } => session.values.get(field.key).map_or("", String::as_str),
    };
    if matches!(row, FormRow::RepeatAdd { .. }) {
        spans.extend(repeat_add_control(locked, focused, rest, styles));
    } else {
        spans.extend(field_control(
            field,
            raw,
            locked,
            focused,
            session.optional_active.contains(field.key),
            rest,
            styles,
        ));
    }
    if gutter_w == 1 {
        let style = if gutter == '▐' {
            styles.key
        } else {
            styles.quiet
        };
        spans.push(Span::styled(gutter.to_string(), style));
    }
    Line::from(spans)
}

fn repeat_add_control(
    locked: bool,
    focused: bool,
    width: usize,
    styles: &Styles,
) -> Vec<Span<'static>> {
    let style = if focused && !locked {
        styles.focus
    } else {
        styles.muted
    };
    vec![Span::styled(pad_visual("+ add", width), style)]
}

fn field_control(
    field: &FieldSpec,
    raw: &str,
    locked: bool,
    focused: bool,
    optional_active: bool,
    width: usize,
    styles: &Styles,
) -> Vec<Span<'static>> {
    let (chrome, value_style) = control_styles(locked, focused, styles);
    if let FieldKind::Optional {
        kind, unset_label, ..
    } = field.kind
    {
        return optional_control(
            kind,
            unset_label,
            field.key,
            raw,
            locked,
            focused,
            optional_active,
            width,
            chrome,
            value_style,
            styles,
        );
    }
    match field.kind {
        FieldKind::Toggle | FieldKind::InvertedToggle | FieldKind::Flag => {
            toggle_control(field.kind.toggle_is_on(raw), locked, focused, width, styles)
        }
        FieldKind::Enum { .. } | FieldKind::LabeledEnum { .. } | FieldKind::Lookup { .. } => {
            combo_control(field, raw, locked, focused, width, chrome, value_style)
        }
        FieldKind::Secret => {
            let shown = secret_input_mask(raw);
            slot_control(
                &shown,
                '[',
                ' ',
                ']',
                focused && !locked,
                locked,
                width,
                chrome,
                value_style,
            )
        }
        FieldKind::Readonly => vec![Span::styled(pad_visual(raw, width), styles.muted)],
        FieldKind::Text
        | FieldKind::Number
        | FieldKind::ConstrainedNumber { .. }
        | FieldKind::Time
        | FieldKind::Ip
        | FieldKind::Ipv6
        | FieldKind::Mac
        | FieldKind::Raw
        | FieldKind::Repeat => slot_control(
            raw,
            '[',
            ' ',
            ']',
            focused && !locked,
            locked,
            width,
            chrome,
            value_style,
        ),
        FieldKind::Optional { .. } => unreachable!("optional handled above"),
    }
}

fn control_styles(locked: bool, focused: bool, styles: &Styles) -> (Style, Style) {
    let chrome = if focused && !locked {
        styles.focus
    } else {
        styles.border
    };
    let value_style = if focused && !locked {
        styles.focus.add_modifier(Modifier::BOLD)
    } else if locked {
        styles.muted
    } else {
        styles.text
    };
    (chrome, value_style)
}

#[allow(clippy::too_many_arguments)]
fn optional_control(
    kind: ScalarKind,
    unset_label: &str,
    field_key: &str,
    raw: &str,
    locked: bool,
    focused: bool,
    optional_active: bool,
    width: usize,
    chrome: Style,
    value_style: Style,
    styles: &Styles,
) -> Vec<Span<'static>> {
    if optional_active {
        return scalar_control(
            kind,
            field_key,
            raw,
            locked,
            focused,
            width,
            chrome,
            value_style,
        );
    }
    vec![Span::styled(
        pad_visual(&format!("+ set ({unset_label})"), width),
        if focused && !locked {
            styles.focus
        } else {
            styles.muted
        },
    )]
}

#[allow(clippy::too_many_arguments)]
fn combo_control(
    field: &FieldSpec,
    raw: &str,
    locked: bool,
    focused: bool,
    width: usize,
    chrome: Style,
    value_style: Style,
) -> Vec<Span<'static>> {
    let typed = field.kind.display_value(raw);
    let shown = if raw.is_empty() {
        "—".to_string()
    } else if typed == raw {
        enum_display_value(field.key, raw)
    } else {
        typed
    };
    slot_control(
        &shown,
        '<',
        '▾',
        '>',
        focused && !locked,
        locked,
        width,
        chrome,
        value_style,
    )
}

#[allow(clippy::too_many_arguments)]
fn scalar_control(
    kind: ScalarKind,
    field_key: &str,
    raw: &str,
    locked: bool,
    focused: bool,
    width: usize,
    chrome: Style,
    value_style: Style,
) -> Vec<Span<'static>> {
    let shown = match kind {
        ScalarKind::Enum { choices } => choices
            .iter()
            .find(|choice| choice.value == raw)
            .map_or_else(|| raw.to_string(), |choice| choice.label.to_string()),
        _ => raw.to_string(),
    };
    let shown = if matches!(kind, ScalarKind::Enum { .. }) && shown.is_empty() {
        "—".to_string()
    } else {
        shown
    };
    let (open, trail, close) = if matches!(kind, ScalarKind::Enum { .. }) {
        ('<', '▾', '>')
    } else {
        ('[', ' ', ']')
    };
    let mut spans = vec![Span::styled("− ".to_string(), chrome)];
    let shown = if shown == raw {
        enum_display_value(field_key, &shown)
    } else {
        shown
    };
    spans.extend(slot_control(
        &shown,
        open,
        trail,
        close,
        focused && !locked,
        locked,
        width.saturating_sub(2),
        chrome,
        value_style,
    ));
    spans
}

fn toggle_control(
    on: bool,
    locked: bool,
    focused: bool,
    width: usize,
    styles: &Styles,
) -> Vec<Span<'static>> {
    let mark = if on { "[x]" } else { "[ ]" };
    let word = if on { "yes" } else { "no" };
    let mark_style = if focused && !locked {
        styles.focus
    } else if on {
        styles.signal
    } else {
        styles.muted
    };
    let word_style = if focused && !locked {
        styles.focus
    } else {
        styles.muted
    };
    let gap = "  ";
    let used = mark.width() + gap.len() + word.width();
    vec![
        Span::styled(mark.to_string(), mark_style),
        Span::styled(
            format!("{gap}{word}{}", " ".repeat(width.saturating_sub(used))),
            word_style,
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn slot_control(
    value: &str,
    open: char,
    trail: char,
    close: char,
    caret: bool,
    locked: bool,
    width: usize,
    chrome: Style,
    value_style: Style,
) -> Vec<Span<'static>> {
    if width < 2 {
        return vec![Span::styled(pad_visual(value, width), value_style)];
    }
    let trail_w = if trail == ' ' {
        0
    } else {
        UnicodeWidthChar::width(trail).unwrap_or(1)
    };
    let inner = width.saturating_sub(2 + trail_w).max(1);
    let mut body = value.to_string();
    if caret {
        body.push('_');
    }
    let padded = pad_visual(&body, inner);
    let suffix = if locked {
        let trimmed = padded.trim_end();
        let note = " locked";
        if trimmed.width() + note.len() <= inner {
            pad_visual(&format!("{trimmed}{note}"), inner)
        } else {
            padded
        }
    } else {
        padded
    };
    let mut spans = vec![
        Span::styled(open.to_string(), chrome),
        Span::styled(suffix, value_style),
    ];
    if trail != ' ' {
        spans.push(Span::styled(trail.to_string(), chrome));
    }
    spans.push(Span::styled(close.to_string(), chrome));
    spans
}

fn pad_visual(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let current = value.width();
    if current > width {
        return clip_line(value, width);
    }
    format!("{value}{}", " ".repeat(width - current))
}
