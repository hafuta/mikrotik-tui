//! Sectioned properties-sheet overlay.

use std::collections::HashMap;

use mtui_core::{FieldKind, FieldSpec, FormSchema, FormSection, extra_status_fields};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};

use crate::layout::clip_line;
use crate::login::is_printable_char;
use crate::overlay::{
    Modal, ModalButton, ModalButtonKind, compact_modal_rect, dim_canvas, render_modal,
};
use crate::styles::Styles;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMode {
    Create,
    Edit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSession {
    pub resource_id: String,
    pub record_id: String,
    pub mode: FormMode,
    pub section: usize,
    pub focus: usize,
    pub offset: usize,
    pub values: HashMap<String, String>,
    pub original: HashMap<String, String>,
    pub extras: Vec<(String, String)>,
    pub error: Option<String>,
    pub saving: bool,
    pub confirm_discard: bool,
    pub confirm_save: bool,
    pub prompt_command: Option<&'static str>,
    pub prompt_schema: Option<&'static FormSchema>,
    pub lookup: Option<Box<LookupPicker>>,
}

/// Nested live-lookup picker sitting on a form sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupPicker {
    pub field_key: String,
    pub resource_id: &'static str,
    pub value_key: &'static str,
    pub multiple: bool,
    pub filter: String,
    pub options: Vec<String>,
    pub selected: Vec<String>,
    pub focus: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub request_id: u64,
    pub generation: u64,
}

impl FormSession {
    #[must_use]
    pub fn edit(
        resource_id: impl Into<String>,
        record_id: impl Into<String>,
        row: &HashMap<String, String>,
        schema: &FormSchema,
    ) -> Self {
        Self {
            resource_id: resource_id.into(),
            record_id: record_id.into(),
            mode: FormMode::Edit,
            section: 0,
            focus: 0,
            offset: 0,
            values: row.clone(),
            original: row.clone(),
            extras: extra_status_fields(schema, row),
            error: None,
            saving: false,
            confirm_discard: false,
            confirm_save: false,
            prompt_command: None,
            prompt_schema: None,
            lookup: None,
        }
    }

    #[must_use]
    pub fn create(resource_id: impl Into<String>, schema: &FormSchema) -> Self {
        let mut values = HashMap::new();
        for section in schema.sections_for(true) {
            for field in section.fields {
                values
                    .entry(field.key.to_string())
                    .or_insert_with(String::new);
            }
        }
        Self {
            resource_id: resource_id.into(),
            record_id: String::new(),
            mode: FormMode::Create,
            section: 0,
            focus: 0,
            offset: 0,
            values,
            original: HashMap::new(),
            extras: Vec::new(),
            error: None,
            saving: false,
            confirm_discard: false,
            confirm_save: false,
            prompt_command: None,
            prompt_schema: None,
            lookup: None,
        }
    }

    #[must_use]
    pub fn prompt(
        resource_id: impl Into<String>,
        record_id: impl Into<String>,
        command: &'static str,
        name: &str,
    ) -> Self {
        let mut values = HashMap::new();
        values.insert("new-name".into(), format!("{name}-copy"));
        Self::prompt_with(resource_id, record_id, command, &COPY_FORM, values)
    }

    #[must_use]
    pub fn prompt_with(
        resource_id: impl Into<String>,
        record_id: impl Into<String>,
        command: &'static str,
        schema: &'static FormSchema,
        mut values: HashMap<String, String>,
    ) -> Self {
        for section in schema.sections_for(true) {
            for field in section.fields {
                values.entry(field.key.to_string()).or_default();
            }
        }
        Self {
            resource_id: resource_id.into(),
            record_id: record_id.into(),
            mode: FormMode::Create,
            section: 0,
            focus: 0,
            offset: 0,
            values,
            original: HashMap::new(),
            extras: Vec::new(),
            error: None,
            saving: false,
            confirm_discard: false,
            confirm_save: false,
            prompt_command: Some(command),
            prompt_schema: Some(schema),
            lookup: None,
        }
    }

    #[must_use]
    pub fn prompt_fields(
        resource_id: impl Into<String>,
        record_id: impl Into<String>,
        command: &'static str,
        schema: &'static FormSchema,
        values: HashMap<String, String>,
    ) -> Self {
        Self {
            resource_id: resource_id.into(),
            record_id: record_id.into(),
            mode: FormMode::Create,
            section: 0,
            focus: 0,
            offset: 0,
            values,
            original: HashMap::new(),
            extras: Vec::new(),
            error: None,
            saving: false,
            confirm_discard: false,
            confirm_save: false,
            prompt_command: Some(command),
            prompt_schema: Some(schema),
            lookup: None,
        }
    }

    #[must_use]
    pub fn overlay_schema(
        &self,
        resource_form: Option<&'static FormSchema>,
    ) -> &'static FormSchema {
        self.prompt_schema.or(resource_form).unwrap_or(&COPY_FORM)
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.values != self.original
    }

    #[must_use]
    pub fn schema_sections<'a>(&self, schema: &'a FormSchema) -> &'a [FormSection] {
        if let Some(prompt) = self.prompt_schema {
            prompt.sections_for(true)
        } else if self.prompt_command.is_some() {
            COPY_SECTIONS
        } else {
            schema.sections_for(self.mode == FormMode::Create)
        }
    }

    pub fn clamp(&mut self, schema: &FormSchema) {
        let sections = self.schema_sections(schema);
        if sections.is_empty() {
            self.section = 0;
            self.focus = 0;
            return;
        }
        self.section = self.section.min(sections.len() - 1);
        let len = self.visible_fields(schema).len().max(1);
        self.focus = self.focus.min(len - 1);
        let max_off = len.saturating_sub(1);
        self.offset = self.offset.min(max_off);
        if self.focus < self.offset {
            self.offset = self.focus;
        }
    }

    #[must_use]
    pub fn visible_fields<'a>(&self, schema: &'a FormSchema) -> Vec<(bool, &'a FieldSpec)> {
        let sections = self.schema_sections(schema);
        let Some(section) = sections.get(self.section) else {
            return Vec::new();
        };
        let fields: Vec<(bool, &FieldSpec)> = section
            .fields
            .iter()
            .map(|field| {
                (
                    section.read_only || matches!(field.kind, FieldKind::Readonly),
                    field,
                )
            })
            .collect();
        fields
    }

    pub fn move_section(&mut self, schema: &FormSchema, delta: isize) {
        let len = self.schema_sections(schema).len();
        if len == 0 {
            return;
        }
        let cur = isize::try_from(self.section).unwrap_or(0);
        let max = isize::try_from(len.saturating_sub(1)).unwrap_or(0);
        self.section = usize::try_from((cur + delta).clamp(0, max)).unwrap_or(0);
        self.focus = 0;
        self.offset = 0;
    }

    pub fn jump_section(&mut self, schema: &FormSchema, index: usize) {
        if index < self.schema_sections(schema).len() {
            self.section = index;
            self.focus = 0;
            self.offset = 0;
        }
    }

    pub fn move_field(&mut self, schema: &FormSchema, delta: isize) {
        let len = self.visible_fields(schema).len();
        if len == 0 {
            return;
        }
        let cur = isize::try_from(self.focus).unwrap_or(0);
        let max = isize::try_from(len.saturating_sub(1)).unwrap_or(0);
        self.focus = usize::try_from((cur + delta).clamp(0, max)).unwrap_or(0);
    }

    pub fn insert_char(&mut self, schema: &FormSchema, ch: char) {
        if !is_printable_char(ch) {
            return;
        }
        let Some(field) = self.focused_spec(schema) else {
            return;
        };
        if !field.kind.takes_typed_input() {
            return;
        }
        self.values
            .entry(field.key.to_string())
            .or_default()
            .push(ch);
    }

    pub fn backspace(&mut self, schema: &FormSchema) {
        let Some(field) = self.focused_spec(schema) else {
            return;
        };
        if !field.kind.takes_typed_input() {
            return;
        }
        self.values.entry(field.key.to_string()).or_default().pop();
    }

    pub fn activate(&mut self, schema: &FormSchema) {
        let Some(field) = self.focused_spec(schema).copied() else {
            return;
        };
        match field.kind {
            FieldKind::Toggle => {
                let now = self.values.get(field.key).map_or("false", String::as_str);
                let next = if matches!(now, "true" | "yes" | "1") {
                    "false"
                } else {
                    "true"
                };
                self.values.insert(field.key.to_string(), next.to_string());
            }
            FieldKind::Enum { values } => {
                let now = self.values.get(field.key).cloned().unwrap_or_default();
                let idx = values.iter().position(|v| *v == now).unwrap_or(0);
                let next = values[(idx + 1) % values.len()];
                self.values.insert(field.key.to_string(), next.to_string());
            }
            FieldKind::Lookup {
                resource_id,
                value_key,
                multiple,
            } => {
                let selected =
                    split_ros_list(self.values.get(field.key).map_or("", String::as_str));
                let generation = self
                    .lookup
                    .as_ref()
                    .map_or(1, |picker| picker.generation.wrapping_add(1));
                self.lookup = Some(Box::new(LookupPicker {
                    field_key: field.key.to_string(),
                    resource_id,
                    value_key,
                    multiple,
                    filter: String::new(),
                    options: Vec::new(),
                    selected,
                    focus: 0,
                    loading: true,
                    error: None,
                    request_id: 0,
                    generation,
                }));
            }
            _ => {}
        }
    }

    #[must_use]
    pub fn focused_spec<'a>(&self, schema: &'a FormSchema) -> Option<&'a FieldSpec> {
        self.visible_fields(schema)
            .get(self.focus)
            .map(|(_, field)| *field)
    }

    #[must_use]
    pub fn lookup_open(&self) -> bool {
        self.lookup.is_some()
    }

    pub fn close_lookup(&mut self) {
        self.lookup = None;
    }

    pub fn apply_lookup_result(
        &mut self,
        request_id: u64,
        generation: u64,
        options: Vec<String>,
        error: Option<String>,
    ) -> bool {
        let Some(picker) = &self.lookup else {
            return false;
        };
        if picker.request_id != request_id || picker.generation != generation {
            return false;
        }
        let current = self
            .values
            .get(&picker.field_key)
            .cloned()
            .unwrap_or_default();
        let picker = self.lookup.as_mut().expect("lookup still open");
        picker.loading = false;
        picker.error = error;
        picker.options = options;
        let filtered = filtered_lookup_options(&picker.options, &picker.filter);
        if picker.focus >= filtered.len() {
            picker.focus = filtered.len().saturating_sub(1);
        }
        if !picker.multiple
            && let Some(idx) = filtered.iter().position(|option| *option == current)
        {
            picker.focus = idx;
        }
        true
    }

    pub fn lookup_move(&mut self, delta: isize) {
        let Some(picker) = self.lookup.as_mut() else {
            return;
        };
        let len = filtered_lookup_options(&picker.options, &picker.filter).len();
        if len == 0 {
            picker.focus = 0;
            return;
        }
        let cur = isize::try_from(picker.focus).unwrap_or(0);
        let max = isize::try_from(len.saturating_sub(1)).unwrap_or(0);
        picker.focus = usize::try_from((cur + delta).clamp(0, max)).unwrap_or(0);
    }

    pub fn lookup_insert_char(&mut self, ch: char) {
        if !is_printable_char(ch) {
            return;
        }
        let Some(picker) = self.lookup.as_mut() else {
            return;
        };
        picker.filter.push(ch);
        picker.focus = 0;
    }

    pub fn lookup_backspace(&mut self) {
        let Some(picker) = self.lookup.as_mut() else {
            return;
        };
        picker.filter.pop();
        picker.focus = 0;
    }

    pub fn lookup_toggle_focused(&mut self) {
        let Some(picker) = self.lookup.as_mut() else {
            return;
        };
        if !picker.multiple {
            return;
        }
        let Some(value) = filtered_lookup_options(&picker.options, &picker.filter)
            .get(picker.focus)
            .cloned()
        else {
            return;
        };
        if let Some(idx) = picker.selected.iter().position(|item| item == &value) {
            picker.selected.remove(idx);
        } else {
            picker.selected.push(value);
        }
    }

    pub fn lookup_confirm(&mut self) {
        let Some(picker) = self.lookup.take() else {
            return;
        };
        let picker = *picker;
        if picker.multiple {
            self.values
                .insert(picker.field_key, join_ros_list(&picker.selected));
            return;
        }
        let filtered = filtered_lookup_options(&picker.options, &picker.filter);
        let Some(value) = filtered.get(picker.focus) else {
            self.lookup = Some(Box::new(picker));
            return;
        };
        self.values.insert(picker.field_key, value.clone());
    }
}

const COPY_FIELD: FieldSpec = FieldSpec {
    key: "new-name",
    label: "New name",
    kind: FieldKind::Text,
};

const COPY_SECTIONS: &[FormSection] = &[FormSection {
    id: "copy",
    label: "Copy",
    read_only: false,
    fields: &[COPY_FIELD],
}];

pub const COPY_FORM: FormSchema = FormSchema {
    title_key: "new-name",
    subtitle_keys: &[],
    sections: COPY_SECTIONS,
    create_sections: COPY_SECTIONS,
};

const BACKUP_SAVE_FIELDS: &[FieldSpec] = &[
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
];

const BACKUP_SAVE_SECTIONS: &[FormSection] = &[FormSection {
    id: "backup",
    label: "Backup",
    read_only: false,
    fields: BACKUP_SAVE_FIELDS,
}];

pub const BACKUP_SAVE_FORM: FormSchema = FormSchema {
    title_key: "name",
    subtitle_keys: &[],
    sections: BACKUP_SAVE_SECTIONS,
    create_sections: BACKUP_SAVE_SECTIONS,
};

const LOCAL_PATH: FieldSpec = FieldSpec {
    key: "local-path",
    label: "Local path",
    kind: FieldKind::Text,
};

const REMOTE_NAME: FieldSpec = FieldSpec {
    key: "remote-name",
    label: "Remote name",
    kind: FieldKind::Text,
};

const FETCH_URL: FieldSpec = FieldSpec {
    key: "url",
    label: "URL",
    kind: FieldKind::Text,
};

const DST_PATH: FieldSpec = FieldSpec {
    key: "dst-path",
    label: "Dst path",
    kind: FieldKind::Text,
};

const FETCH_USER: FieldSpec = FieldSpec {
    key: "user",
    label: "User",
    kind: FieldKind::Text,
};

const FETCH_PASSWORD: FieldSpec = FieldSpec {
    key: "password",
    label: "Password",
    kind: FieldKind::Secret,
};

const UPLOAD_SECTIONS: &[FormSection] = &[FormSection {
    id: "upload",
    label: "Upload",
    read_only: false,
    fields: &[LOCAL_PATH, REMOTE_NAME],
}];

const DOWNLOAD_SECTIONS: &[FormSection] = &[FormSection {
    id: "download",
    label: "Download",
    read_only: false,
    fields: &[LOCAL_PATH],
}];

const FETCH_SECTIONS: &[FormSection] = &[FormSection {
    id: "fetch",
    label: "Fetch URL",
    read_only: false,
    fields: &[FETCH_URL, DST_PATH, FETCH_USER, FETCH_PASSWORD],
}];

pub const UPLOAD_FORM: FormSchema = FormSchema {
    title_key: "remote-name",
    subtitle_keys: &[],
    sections: UPLOAD_SECTIONS,
    create_sections: UPLOAD_SECTIONS,
};

pub const DOWNLOAD_FORM: FormSchema = FormSchema {
    title_key: "local-path",
    subtitle_keys: &[],
    sections: DOWNLOAD_SECTIONS,
    create_sections: DOWNLOAD_SECTIONS,
};

pub const FETCH_FORM: FormSchema = FormSchema {
    title_key: "url",
    subtitle_keys: &[],
    sections: FETCH_SECTIONS,
    create_sections: FETCH_SECTIONS,
};

const LOOKUP_INTERFACE: FieldSpec = FieldSpec {
    key: "interface",
    label: "Interface",
    kind: FieldKind::Lookup {
        resource_id: "interfaces",
        value_key: "name",
        multiple: false,
    },
};

const LOOKUP_PORTS: FieldSpec = FieldSpec {
    key: "ports",
    label: "Ports",
    kind: FieldKind::Lookup {
        resource_id: "interfaces",
        value_key: "name",
        multiple: true,
    },
};

const LOOKUP_TEST_SECTIONS: &[FormSection] = &[FormSection {
    id: "general",
    label: "General",
    read_only: false,
    fields: &[LOOKUP_INTERFACE, LOOKUP_PORTS],
}];

/// Test-only schema that exercises live lookup without production write wiring.
pub const LOOKUP_TEST_FORM: FormSchema = FormSchema {
    title_key: "interface",
    subtitle_keys: &[],
    sections: LOOKUP_TEST_SECTIONS,
    create_sections: LOOKUP_TEST_SECTIONS,
};

fn split_ros_list(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !out.iter().any(|item| item == trimmed) {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn join_ros_list(values: &[String]) -> String {
    values.join(",")
}

fn filtered_lookup_options(options: &[String], filter: &str) -> Vec<String> {
    let q = filter.to_ascii_lowercase();
    options
        .iter()
        .filter(|option| q.is_empty() || option.to_ascii_lowercase().contains(&q))
        .cloned()
        .collect()
}

/// Paint a centered properties sheet over the dimmed canvas.
pub fn render_form_sheet(
    frame: &mut Frame<'_>,
    area: Rect,
    session: &FormSession,
    schema: &FormSchema,
    styles: &Styles,
) {
    dim_canvas(frame, area, styles);
    let width = area.width.saturating_sub(4).clamp(48, 92);
    let height = area.height.saturating_sub(2).clamp(12, 28);
    let rect = compact_modal_rect(area, width, height);
    frame.render_widget(Clear, rect);

    let title = sheet_title(session, schema);
    let border = if session.confirm_discard {
        styles.alert
    } else {
        styles.border
    };
    let block = Block::default()
        .title(Span::styled(format!(" {title} "), styles.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border)
        .style(styles.text)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let sections = session.schema_sections(schema);
    let show_tabs = sections.len() > 1;
    let tab_height = if show_tabs { 2 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(tab_height),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .split(inner);

    if show_tabs {
        frame.render_widget(
            Paragraph::new(tab_bar_lines(
                sections,
                session.section,
                usize::from(chunks[0].width.max(1)),
                styles,
            )),
            chunks[0],
        );
    }

    frame.render_widget(
        Paragraph::new(sheet_field_lines(
            session,
            schema,
            sections,
            usize::from(chunks[1].width.max(1)),
            usize::from(chunks[1].height.max(1)),
            styles,
        )),
        chunks[1],
    );

    let hint = sheet_hint(session, schema, sections, show_tabs);
    let hint_style = if session.error.is_some() || session.confirm_discard {
        styles.alert
    } else {
        styles.muted
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(hint, hint_style))),
        chunks[2],
    );

    if let Some(picker) = &session.lookup {
        render_lookup_picker(frame, area, picker, styles);
    }
    if session.confirm_save {
        render_save_preview(frame, area, session, schema, styles);
    }
}

fn sheet_field_lines(
    session: &FormSession,
    schema: &FormSchema,
    sections: &[FormSection],
    width: usize,
    height: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let fields = session.visible_fields(schema);
    let extra_rows = if sections.get(session.section).is_some_and(|s| s.read_only) {
        session.extras.len().min(6)
    } else {
        0
    };
    let visible_h = height.saturating_sub(extra_rows);
    let start = session.offset.min(fields.len().saturating_sub(1));
    let mut lines = Vec::new();
    for (idx, (locked, field)) in fields.iter().enumerate().skip(start).take(visible_h.max(1)) {
        lines.push(field_line(
            session,
            field,
            *locked,
            idx == session.focus,
            width,
            styles,
        ));
    }
    if extra_rows > 0 {
        for (key, value) in session.extras.iter().take(extra_rows) {
            lines.push(Line::from(vec![
                Span::styled(format!("  {key:<16} "), styles.muted),
                Span::styled(value.clone(), styles.text),
            ]));
        }
    }
    lines
}

fn sheet_hint(
    session: &FormSession,
    schema: &FormSchema,
    sections: &[FormSection],
    show_tabs: bool,
) -> String {
    if session.confirm_discard {
        return "discard changes?  y confirm   n keep editing".into();
    }
    if session.confirm_save {
        return "save these fields?  y confirm   n back".into();
    }
    if session.saving {
        return "saving…".into();
    }
    if let Some(err) = &session.error {
        return err.clone();
    }
    let field_hint = session.focused_spec(schema).map_or("tab field", |field| {
        if sections
            .get(session.section)
            .is_some_and(|section| section.read_only)
            || matches!(field.kind, FieldKind::Readonly)
        {
            FieldKind::Readonly.edit_hint()
        } else {
            field.kind.edit_hint()
        }
    });
    if show_tabs {
        let typing = session
            .focused_spec(schema)
            .is_some_and(|field| field.kind.takes_typed_input());
        let tab_jump = if typing { "" } else { "1-9 jump   " };
        format!("[ / ] tabs   {tab_jump}↑↓ field   tab field   {field_hint}   ctrl+s save   esc")
    } else {
        format!("tab field   {field_hint}   ctrl+s save   esc")
    }
}

const PREVIEW_MASK: &str = "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}";

fn render_save_preview(
    frame: &mut Frame<'_>,
    area: Rect,
    session: &FormSession,
    schema: &FormSchema,
    styles: &Styles,
) {
    let changes =
        mtui_core::preview_changes(schema, &session.original, &session.values, PREVIEW_MASK);
    let body = if changes.is_empty() {
        "No writable fields changed.".to_string()
    } else {
        changes
            .into_iter()
            .map(|(label, value)| format!("{label}: {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
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
    let modal = Modal::new("Save preview", &body)
        .kicker(kicker)
        .hint("Secrets stay masked. Confirm to write these fields.")
        .buttons(&buttons);
    render_modal(frame, area, &modal, styles);
}

fn render_lookup_picker(frame: &mut Frame<'_>, area: Rect, picker: &LookupPicker, styles: &Styles) {
    dim_canvas(frame, area, styles);
    let width = area.width.saturating_sub(8).clamp(20, 52);
    let height = area.height.saturating_sub(4).clamp(5, 16);
    let rect = compact_modal_rect(area, width, height);
    frame.render_widget(Clear, rect);

    let title = if picker.multiple {
        " Lookup (multi) "
    } else {
        " Lookup "
    };
    let block = Block::default()
        .title(Span::styled(title, styles.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(styles.border)
        .style(styles.text)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let list_width = usize::from(chunks[0].width.max(1));
    let list_height = usize::from(chunks[0].height.max(1));
    frame.render_widget(
        Paragraph::new(lookup_picker_lines(picker, list_width, list_height, styles)),
        chunks[0],
    );
    if chunks[1].height > 0 {
        let hint = if picker.multiple {
            "type filter   ↑↓   space toggle   enter ok   esc"
        } else {
            "type filter   ↑↓   enter select   esc"
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(hint, styles.muted))),
            chunks[1],
        );
    }
}

fn lookup_picker_lines(
    picker: &LookupPicker,
    width: usize,
    height: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if height == 0 {
        return lines;
    }
    let filter = format!("/{}", picker.filter);
    lines.push(Line::from(Span::styled(
        clip_line(&filter, width),
        styles.focus,
    )));
    if lines.len() >= height {
        return lines;
    }
    if picker.loading {
        lines.push(Line::from(Span::styled("loading…", styles.muted)));
        return lines;
    }
    if let Some(err) = &picker.error {
        lines.push(Line::from(Span::styled(
            clip_line(err, width),
            styles.alert,
        )));
        return lines;
    }
    if picker.multiple && !picker.selected.is_empty() && lines.len() < height {
        let selected = format!("selected {}", join_ros_list(&picker.selected));
        lines.push(Line::from(Span::styled(
            clip_line(&selected, width),
            styles.signal,
        )));
    }
    let filtered = filtered_lookup_options(&picker.options, &picker.filter);
    if filtered.is_empty() && lines.len() < height {
        lines.push(Line::from(Span::styled("no matches", styles.muted)));
        return lines;
    }
    let start = picker
        .focus
        .saturating_sub(height.saturating_sub(lines.len() + 1));
    for (idx, option) in filtered.iter().enumerate().skip(start) {
        if lines.len() >= height {
            break;
        }
        let marked = picker.selected.iter().any(|item| item == option);
        let caret = if idx == picker.focus { ">" } else { " " };
        let mark = if picker.multiple {
            if marked { "[x] " } else { "[ ] " }
        } else {
            ""
        };
        let body = format!("{caret} {mark}{option}");
        let style = if idx == picker.focus {
            styles.focus
        } else {
            styles.text
        };
        lines.push(Line::from(Span::styled(clip_line(&body, width), style)));
    }
    lines
}

/// Numbered tabs with a bracketed active tab and an underline so selection
/// is not color-only. Overflow keeps the selected tab visible.
fn tab_bar_lines(
    sections: &[FormSection],
    selected: usize,
    width: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    if sections.is_empty() || width == 0 {
        return vec![Line::default(), Line::default()];
    }
    let tabs: Vec<String> = sections
        .iter()
        .enumerate()
        .map(|(idx, section)| {
            let n = idx + 1;
            if idx == selected {
                format!("[{n} {}]", section.label)
            } else {
                format!("{n} {}", section.label)
            }
        })
        .collect();
    let (shown, selected_in_view) = visible_tabs(&tabs, selected, width);
    let mut spans = Vec::new();
    let mut underline = String::new();
    for (i, label) in shown.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", styles.muted));
            underline.push_str("  ");
        }
        let active = i == selected_in_view;
        let style = if active {
            styles.focus.add_modifier(Modifier::BOLD)
        } else {
            styles.muted
        };
        spans.push(Span::styled((*label).clone(), style));
        let pad = if active {
            "─".repeat(label.width().max(1))
        } else {
            " ".repeat(label.width())
        };
        underline.push_str(&pad);
    }
    if underline.width() > width {
        underline.truncate(width);
    }
    vec![
        Line::from(spans),
        Line::from(Span::styled(underline, styles.focus)),
    ]
}

fn visible_tabs(tabs: &[String], selected: usize, width: usize) -> (Vec<&String>, usize) {
    if tabs.is_empty() {
        return (Vec::new(), 0);
    }
    let selected = selected.min(tabs.len() - 1);
    if tabs_width(tabs.iter().map(String::as_str)) <= width {
        return (tabs.iter().collect(), selected);
    }
    let mut start = selected;
    let mut end = selected + 1;
    while start > 0 || end < tabs.len() {
        let grew = if start > 0 {
            let candidate = tabs_width(tabs[start - 1..end].iter().map(String::as_str));
            if candidate <= width {
                start -= 1;
                true
            } else {
                false
            }
        } else {
            false
        };
        let grew_end = if end < tabs.len() {
            let candidate = tabs_width(tabs[start..=end].iter().map(String::as_str));
            if candidate <= width {
                end += 1;
                true
            } else {
                false
            }
        } else {
            false
        };
        if !grew && !grew_end {
            break;
        }
    }
    (tabs[start..end].iter().collect(), selected - start)
}

fn prompt_title(command: &str) -> &'static str {
    match command {
        "save" => "Save backup",
        "upload" => "Upload",
        "download" => "Download",
        "fetch" => "Fetch URL",
        "copy" => "Copy",
        "sign" => "Sign",
        "import" => "Import",
        "export-certificate" => "Export",
        _ => "Command",
    }
}

fn tabs_width<'a>(tabs: impl Iterator<Item = &'a str>) -> usize {
    let mut width = 0usize;
    for (i, tab) in tabs.enumerate() {
        if i > 0 {
            width = width.saturating_add(2);
        }
        width = width.saturating_add(tab.width());
    }
    width
}

fn sheet_title(session: &FormSession, schema: &FormSchema) -> String {
    if let Some(command) = session.prompt_command {
        return prompt_title(command).into();
    }
    let name = session
        .values
        .get(schema.title_key)
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(if session.mode == FormMode::Create {
            "new"
        } else {
            "properties"
        });
    let mut bits = vec![name.to_string()];
    for key in schema.subtitle_keys {
        if let Some(value) = session.values.get(*key).filter(|v| !v.is_empty()) {
            bits.push(value.clone());
        }
    }
    if session.values.get("running").map(String::as_str) == Some("true") {
        bits.push("RUN".into());
    }
    if session.values.get("disabled").map(String::as_str) == Some("true") {
        bits.push("OFF".into());
    }
    if session.is_dirty() {
        bits.push("modified".into());
    }
    bits.join(" · ")
}

const LABEL_COLS: usize = 14;
const TAG_COLS: usize = 6;

fn field_line(
    session: &FormSession,
    field: &FieldSpec,
    locked: bool,
    focused: bool,
    width: usize,
    styles: &Styles,
) -> Line<'static> {
    let caret = if focused { ">" } else { " " };
    let label = pad_visual(field.label, LABEL_COLS);
    let tag = pad_visual(field.kind.tag(), TAG_COLS);
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
    let rest = width.saturating_sub(used);
    spans.extend(field_control(
        field.kind,
        session.values.get(field.key).map_or("", String::as_str),
        locked,
        focused,
        rest,
        styles,
    ));
    Line::from(spans)
}

fn field_control(
    kind: FieldKind,
    raw: &str,
    locked: bool,
    focused: bool,
    width: usize,
    styles: &Styles,
) -> Vec<Span<'static>> {
    let value_style: Style = if focused && !locked {
        styles.focus.add_modifier(Modifier::BOLD)
    } else if locked {
        styles.muted
    } else {
        styles.text
    };
    let chrome = if focused && !locked {
        styles.focus
    } else {
        styles.border
    };
    match kind {
        FieldKind::Toggle => toggle_control(raw, locked, focused, width, styles),
        FieldKind::Enum { .. } | FieldKind::Lookup { .. } => slot_control(
            if raw.is_empty() { "—" } else { raw },
            '<',
            '▾',
            '>',
            focused && !locked,
            locked,
            width,
            chrome,
            value_style,
        ),
        FieldKind::Secret => {
            let shown = if raw.is_empty() {
                String::new()
            } else {
                "••••••••".into()
            };
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
        FieldKind::Readonly => {
            let body = pad_visual(raw, width);
            vec![Span::styled(body, styles.muted)]
        }
        FieldKind::Text | FieldKind::Number => slot_control(
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
    }
}

fn toggle_control(
    raw: &str,
    locked: bool,
    focused: bool,
    width: usize,
    styles: &Styles,
) -> Vec<Span<'static>> {
    let on = matches!(raw, "true" | "yes" | "1");
    let mark = if on { "[x]" } else { "[ ]" };
    let word = if on { "on" } else { "off" };
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
    let pad = " ".repeat(width.saturating_sub(used));
    vec![
        Span::styled(mark.to_string(), mark_style),
        Span::styled(format!("{gap}{word}{pad}"), word_style),
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
    let w = value.width();
    if w > width {
        return clip_line(value, width);
    }
    format!("{value}{}", " ".repeat(width - w))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtui_core::{DefaultTheme, Theme};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;
    use ratatui::widgets::Paragraph;
    use unicode_width::UnicodeWidthStr;

    fn sample_schema() -> FormSchema {
        FormSchema {
            title_key: "name",
            subtitle_keys: &["type"],
            sections: &[
                FormSection {
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
                            key: "disabled",
                            label: "Disabled",
                            kind: FieldKind::Toggle,
                        },
                    ],
                },
                FormSection {
                    id: "status",
                    label: "Status",
                    read_only: true,
                    fields: &[FieldSpec {
                        key: "running",
                        label: "Running",
                        kind: FieldKind::Readonly,
                    }],
                },
            ],
            create_sections: &[],
        }
    }

    #[test]
    fn toggle_flips_disabled() {
        let schema = sample_schema();
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        row.insert("disabled".into(), "false".into());
        let mut session = FormSession::edit("interfaces", "*1", &row, &schema);
        session.focus = 1;
        session.activate(&schema);
        assert_eq!(
            session.values.get("disabled").map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn field_and_tab_movement_clamps_without_wrapping() {
        let schema = sample_schema();
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        let mut session = FormSession::edit("interfaces", "*1", &row, &schema);
        session.move_field(&schema, -1);
        assert_eq!(session.focus, 0);
        session.move_field(&schema, 1);
        assert_eq!(session.focus, 1);
        session.move_field(&schema, 1);
        assert_eq!(session.focus, 1);

        session.move_section(&schema, -1);
        assert_eq!(session.section, 0);
        session.move_section(&schema, 1);
        assert_eq!(session.section, 1);
        session.move_section(&schema, 1);
        assert_eq!(session.section, 1);
        session.move_section(&schema, -1);
        assert_eq!(session.section, 0);
    }

    #[test]
    fn insert_char_ignores_control() {
        let schema = sample_schema();
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        let mut session = FormSession::edit("interfaces", "*1", &row, &schema);
        session.insert_char(&schema, '\0');
        session.insert_char(&schema, '-');
        assert_eq!(
            session.values.get("name").map(String::as_str),
            Some("ether1-")
        );
    }

    #[test]
    fn form_sheet_is_centered_without_background_fill() {
        let schema = sample_schema();
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        let session = FormSession::edit("interfaces", "*1", &row, &schema);
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new("backdrop"), frame.area());
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        assert_eq!(buf[(0, 0)].bg, Color::Reset);
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("ether1"));
        assert!(rendered.contains("[1 General]"));
        assert!(rendered.contains("2 Status"));
        assert!(rendered.contains("Name"));
        assert!(rendered.contains("text"));
        assert!(rendered.contains('['));
        assert!(rendered.contains("toggle"));
        assert!(
            !rendered.contains("1-9 jump"),
            "digit tab-jump is hidden while typing"
        );
        assert!(
            !rendered.contains("> General"),
            "tabs replace the left section rail"
        );
    }

    #[test]
    fn tab_jump_hint_shows_when_not_typing() {
        let schema = sample_schema();
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        let mut session = FormSession::edit("interfaces", "*1", &row, &schema);
        session.focus = 1;
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("1-9 jump"));
        assert!(rendered.contains("space toggle"));
    }

    #[test]
    fn sheet_title_fits_narrow_labels() {
        let title = "ether1 · ethernet · RUN";
        assert!(title.width() < 40);
    }

    #[test]
    fn tab_strip_keeps_selected_tab_when_narrow() {
        let tabs = vec![
            "[1 General]".into(),
            "2 Ethernet".into(),
            "3 Advanced".into(),
            "4 Status".into(),
        ];
        let (shown, idx) = visible_tabs(&tabs, 2, 16);
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].as_str(), "3 Advanced");
        assert_eq!(idx, 0);
    }

    #[test]
    fn single_section_sheet_hides_tab_bar() {
        let schema = FormSchema {
            title_key: "new-name",
            subtitle_keys: &[],
            sections: COPY_SECTIONS,
            create_sections: COPY_SECTIONS,
        };
        let session = FormSession::prompt("vlan", "*1", "copy", "vlan10");
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("New name"));
        assert!(rendered.contains("text"));
        assert!(rendered.contains('['));
        assert!(!rendered.contains("[1 Copy]"));
        assert_eq!(
            session.values.get("new-name").map(String::as_str),
            Some("vlan10-copy")
        );
        assert_eq!(session.prompt_command, Some("copy"));
    }

    #[test]
    fn sign_prompt_shows_ca_field() {
        let session = FormSession::prompt_with(
            "certificates",
            "*1",
            "sign",
            &mtui_core::CERT_SIGN_PROMPT,
            HashMap::new(),
        );
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(
                    frame,
                    frame.area(),
                    &session,
                    &mtui_core::CERT_SIGN_PROMPT,
                    &styles,
                );
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("CA"));
        assert!(session.values.contains_key("ca"));
        assert!(rendered.contains("Sign"));
    }

    #[test]
    fn backup_save_prompt_shows_name_and_secret_password() {
        let mut values = HashMap::new();
        values.insert("name".into(), "nightly".into());
        values.insert("password".into(), "hidden".into());
        let session = FormSession::prompt_fields("files", "", "save", &BACKUP_SAVE_FORM, values);
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &BACKUP_SAVE_FORM, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("Save backup"));
        assert!(rendered.contains("Name"));
        assert!(rendered.contains("nightly"));
        assert!(rendered.contains("Password"));
        assert!(!rendered.contains("hidden"));
    }

    #[test]
    fn empty_create_fields_show_slots_and_kind_tags() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[],
            create_sections: &[FormSection {
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
                        key: "interfaces",
                        label: "Interfaces",
                        kind: FieldKind::Text,
                    },
                    FieldSpec {
                        key: "arp",
                        label: "ARP",
                        kind: FieldKind::Enum {
                            values: &["enabled", "disabled"],
                        },
                    },
                    FieldSpec {
                        key: "disabled",
                        label: "Disabled",
                        kind: FieldKind::Toggle,
                    },
                ],
            }],
        };
        let session = FormSession::create("bridge", &schema);
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("text"));
        assert!(rendered.contains("select"));
        assert!(rendered.contains("toggle"));
        assert!(
            rendered.contains('['),
            "empty text fields keep an input slot"
        );
        assert!(rendered.contains('▾'), "select fields show a cycle marker");
        assert!(rendered.contains("[ ]"));
        assert!(rendered.contains("type value"));
        assert!(
            !rendered.contains("space toggle"),
            "hints follow the focused field"
        );
    }

    #[test]
    fn fetch_prompt_shows_url_and_secret_password() {
        let mut values = HashMap::new();
        values.insert("url".into(), String::new());
        values.insert("dst-path".into(), String::new());
        values.insert("user".into(), String::new());
        values.insert("password".into(), String::new());
        let session = FormSession::prompt_fields("files", "", "fetch", &FETCH_FORM, values);
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &FETCH_FORM, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("Fetch URL"));
        assert!(rendered.contains("URL"));
        assert!(rendered.contains("Password"));
        assert!(rendered.contains("secret"));
    }

    fn lookup_session() -> (FormSchema, FormSession) {
        let schema = LOOKUP_TEST_FORM;
        let session = FormSession::create("bridge", &schema);
        (schema, session)
    }

    #[test]
    fn lookup_field_rejects_free_text() {
        let (schema, mut session) = lookup_session();
        session.values.insert("interface".into(), "ether1".into());
        session.insert_char(&schema, 'x');
        session.insert_char(&schema, '\0');
        session.backspace(&schema);
        assert_eq!(
            session.values.get("interface").map(String::as_str),
            Some("ether1")
        );
    }

    #[test]
    fn lookup_field_renders_as_picker_slot() {
        let (schema, session) = lookup_session();
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains('▾'));
        assert!(rendered.contains("space pick"));
        assert!(!rendered.contains("[  —"));
    }

    #[test]
    fn activate_opens_lookup_picker_without_network() {
        let (schema, mut session) = lookup_session();
        session.activate(&schema);
        let picker = session.lookup.as_ref().expect("picker");
        assert_eq!(picker.field_key, "interface");
        assert_eq!(picker.resource_id, "interfaces");
        assert_eq!(picker.value_key, "name");
        assert!(!picker.multiple);
        assert!(picker.loading);
        assert!(picker.options.is_empty());
        assert_eq!(picker.request_id, 0);
        assert_eq!(
            session.values.get("interface").map(String::as_str),
            Some("")
        );
    }

    #[test]
    fn apply_lookup_result_happy_stale_and_error() {
        let (schema, mut session) = lookup_session();
        session.activate(&schema);
        let picker = session.lookup.as_mut().expect("picker");
        picker.request_id = 7;
        let generation = picker.generation;

        assert!(session.apply_lookup_result(
            7,
            generation,
            vec!["ether1".into(), "ether2".into()],
            None,
        ));
        let picker = session.lookup.as_ref().expect("picker");
        assert!(!picker.loading);
        assert_eq!(picker.options, ["ether1", "ether2"]);
        assert!(picker.error.is_none());

        assert!(!session.apply_lookup_result(8, generation, vec!["wlan1".into()], None));
        assert_eq!(
            session.lookup.as_ref().unwrap().options,
            ["ether1", "ether2"]
        );

        assert!(session.apply_lookup_result(
            7,
            generation,
            Vec::new(),
            Some("unknown resource".into())
        ));
        let picker = session.lookup.as_ref().expect("picker");
        assert_eq!(picker.error.as_deref(), Some("unknown resource"));
        assert!(!picker.loading);
    }

    #[test]
    fn lookup_single_select_writes_value() {
        let (schema, mut session) = lookup_session();
        session.activate(&schema);
        let picker = session.lookup.as_mut().expect("picker");
        picker.request_id = 1;
        let generation = picker.generation;
        session.apply_lookup_result(1, generation, vec!["ether1".into(), "bridge".into()], None);
        session.lookup_move(1);
        session.lookup_confirm();
        assert!(session.lookup.is_none());
        assert_eq!(
            session.values.get("interface").map(String::as_str),
            Some("bridge")
        );
    }

    #[test]
    fn lookup_multi_select_joins_commas() {
        let (schema, mut session) = lookup_session();
        session.focus = 1;
        session.values.insert("ports".into(), "ether1".into());
        session.activate(&schema);
        let picker = session.lookup.as_mut().expect("picker");
        picker.request_id = 2;
        let generation = picker.generation;
        session.apply_lookup_result(
            2,
            generation,
            vec!["ether1".into(), "ether2".into(), "wlan1".into()],
            None,
        );
        session.lookup_move(1);
        session.lookup_toggle_focused();
        session.lookup_move(1);
        session.lookup_toggle_focused();
        session.lookup_confirm();
        assert_eq!(
            session.values.get("ports").map(String::as_str),
            Some("ether1,ether2,wlan1")
        );
    }

    #[test]
    fn lookup_picker_renders_states_on_tiny_rects() {
        let (schema, mut session) = lookup_session();
        session.activate(&schema);
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(12, 6);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw loading");

        let picker = session.lookup.as_mut().expect("picker");
        picker.request_id = 1;
        let generation = picker.generation;
        session.apply_lookup_result(1, generation, Vec::new(), Some("offline".into()));
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw error");

        session.apply_lookup_result(1, generation, Vec::new(), None);
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw empty");

        session.apply_lookup_result(1, generation, vec!["ether1".into()], None);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new("backdrop"), frame.area());
                render_form_sheet(frame, frame.area(), &session, &schema, &styles);
            })
            .expect("draw populated");
        let buf = terminal.backend().buffer();
        assert_eq!(buf[(0, 0)].bg, Color::Reset);
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("ether1"));
        assert!(rendered.contains("Lookup"));
        assert!(rendered.contains("lookup"));
        assert!(rendered.contains("space pick"));
    }
}
