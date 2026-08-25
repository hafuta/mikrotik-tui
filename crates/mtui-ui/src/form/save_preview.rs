//! Save-preview state and modal rendering.

use mtui_core::FormSchema;
use ratatui::{Frame, layout::Rect};

use crate::overlay::{Modal, ModalButton, ModalButtonKind, render_modal};
use crate::styles::Styles;

use super::{FormMode, FormSession};

const PREVIEW_MASK: &str = "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum SavePreviewState {
    #[default]
    Ready,
    Pending,
    Failed,
}

pub(super) fn render_save_preview(
    frame: &mut Frame<'_>,
    area: Rect,
    session: &FormSession,
    schema: &FormSchema,
    styles: &Styles,
) {
    let changes = mtui_core::preview_changes(
        &session.resource_id,
        schema,
        &session.original,
        &session.values,
        PREVIEW_MASK,
    );
    let changes = if changes.is_empty() {
        "No writable fields changed.".to_string()
    } else {
        changes
            .into_iter()
            .map(|(label, value)| format!("{label}: {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    match session.save_preview_state {
        SavePreviewState::Ready => render_ready(frame, area, session, &changes, styles),
        SavePreviewState::Pending => render_pending(frame, area, &changes, styles),
        SavePreviewState::Failed => render_failed(frame, area, session, &changes, styles),
    }
}

fn render_ready(
    frame: &mut Frame<'_>,
    area: Rect,
    session: &FormSession,
    changes: &str,
    styles: &Styles,
) {
    let buttons = [
        ModalButton {
            label: "Save",
            keys: "y / enter / ctrl+s",
            kind: ModalButtonKind::Primary,
        },
        ModalButton {
            label: "Back",
            keys: "n / esc",
            kind: ModalButtonKind::Secondary,
        },
    ];
    let kicker = if session.mode == FormMode::Create {
        "Fields that will be created"
    } else {
        "Changed fields only"
    };
    let modal = Modal::new("Save preview", changes)
        .kicker(kicker)
        .hint("Secrets stay masked. Confirm to write these fields.")
        .buttons(&buttons);
    render_modal(frame, area, &modal, styles);
}

fn render_pending(frame: &mut Frame<'_>, area: Rect, changes: &str, styles: &Styles) {
    let modal = Modal::new("Saving changes", changes)
        .kicker("WRITE IN PROGRESS")
        .hint("Waiting for RouterOS. The form remains open until the write completes.");
    render_modal(frame, area, &modal, styles);
}

fn render_failed(
    frame: &mut Frame<'_>,
    area: Rect,
    session: &FormSession,
    changes: &str,
    styles: &Styles,
) {
    let error = session
        .error
        .as_deref()
        .unwrap_or("RouterOS rejected the write.");
    let body = format!("Error\n{error}\n\nChanges kept for retry:\n{changes}");
    let buttons = [
        ModalButton {
            label: "Retry",
            keys: "y / enter / ctrl+s",
            kind: ModalButtonKind::Primary,
        },
        ModalButton {
            label: "Back",
            keys: "n / esc",
            kind: ModalButtonKind::Secondary,
        },
    ];
    let modal = Modal::new("Save failed", &body)
        .alert()
        .kicker("ROUTEROS REJECTED THE SAVE")
        .accent_heading("Error")
        .hint("Retry the same values, or go Back to edit them.")
        .buttons(&buttons);
    render_modal(frame, area, &modal, styles);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, FieldKind, FieldSpec, FormSection, Theme};
    use ratatui::{Terminal, backend::TestBackend};

    fn rendered_lines(terminal: &Terminal<TestBackend>) -> Vec<String> {
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn failed_save_renders_error_in_top_modal_and_masks_secrets() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "name",
                        label: "Name",
                        kind: FieldKind::Text,
                    },
                    FieldSpec {
                        key: "password",
                        label: "Password",
                        kind: FieldKind::Secret,
                    },
                ],
            }],
            create_sections: &[],
        };
        let mut original = std::collections::HashMap::new();
        original.insert("name".into(), "old".into());
        original.insert("password".into(), "old-secret".into());
        let mut session = FormSession::edit("test", "*1", &original, &schema);
        session.values.insert("name".into(), "new".into());
        session.values.insert("password".into(), "hunter2".into());
        session.open_save_preview();
        session.begin_save();

        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(64, 18);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                super::super::render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw pending");
        let pending = rendered_lines(&terminal).join("\n");
        assert!(pending.contains("Saving changes"));
        assert!(pending.contains("WRITE IN PROGRESS"));
        assert!(pending.contains(PREVIEW_MASK));
        assert!(!pending.contains("hunter2"));

        session.apply_mutation_error(
            "permission denied while applying the requested RouterOS changes; check write policy"
                .into(),
        );
        terminal
            .draw(|frame| {
                super::super::render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw");

        let lines = rendered_lines(&terminal);
        let rendered = lines.join("\n");
        let title_row = lines
            .iter()
            .position(|line| line.contains("Save failed"))
            .expect("failed modal title");
        let error_row = lines
            .iter()
            .position(|line| line.contains("permission denied"))
            .expect("inline modal error");
        assert!(error_row > title_row, "error must be inside the top modal");
        assert!(rendered.contains("Retry"));
        assert!(rendered.contains("Back"));
        assert!(rendered.contains(PREVIEW_MASK));
        assert!(!rendered.contains("hunter2"));
        assert!(!rendered.contains("old-secret"));
    }
}
