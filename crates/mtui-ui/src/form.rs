//! Sectioned properties-sheet overlay.

use std::collections::{HashMap, HashSet};

use mtui_core::{
    FieldKind, FieldSpec, FormSchema, FormSection, ScalarKind, default_writable_value,
    extra_status_fields, field_enabled, field_visible, join_ros_list, prepare_lookup_options,
    split_ros_list, with_leading_none,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};

use crate::layout::clip_line;
use crate::login::is_printable_char;
use crate::overlay::{compact_modal_rect, dim_canvas};
use crate::scroll::ScrollView;
use crate::styles::Styles;
use unicode_width::UnicodeWidthStr;

mod layout;
mod navigation;
mod rows;
mod save_preview;

use save_preview::{SavePreviewState, render_save_preview};

const SHEET_HINT_ROWS: u16 = 1;

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
    save_preview_state: SavePreviewState,
    mutation_request_id: Option<u64>,
    /// In-flight typed-source print (Ethernet `poe-*` / `sfp-*` from `/interface/ethernet`).
    pub hydrate_request_id: Option<u64>,
    pub prompt_command: Option<&'static str>,
    pub prompt_schema: Option<&'static FormSchema>,
    pub lookup: Option<Box<LookupPicker>>,
    /// In-progress repeater rows (may include empty drafts not yet in `values`).
    pub repeat: HashMap<String, Vec<String>>,
    /// Optional scalar fields currently expanded into an input.
    pub optional_active: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormRow<'a> {
    Field {
        locked: bool,
        field: &'a FieldSpec,
    },
    RepeatItem {
        locked: bool,
        field: &'a FieldSpec,
        index: usize,
    },
    RepeatAdd {
        locked: bool,
        field: &'a FieldSpec,
    },
}

impl<'a> FormRow<'a> {
    fn field(self) -> &'a FieldSpec {
        match self {
            Self::Field { field, .. }
            | Self::RepeatItem { field, .. }
            | Self::RepeatAdd { field, .. } => field,
        }
    }

    fn locked(self) -> bool {
        match self {
            Self::Field { locked, .. }
            | Self::RepeatItem { locked, .. }
            | Self::RepeatAdd { locked, .. } => locked,
        }
    }
}

fn rows_for_section<'a>(session: &FormSession, section: &'a FormSection) -> Vec<FormRow<'a>> {
    let mut rows = Vec::new();
    for field in section
        .fields
        .iter()
        .filter(|field| field_visible(&session.resource_id, field.key, &session.values))
    {
        let locked = section.read_only
            || matches!(field.kind, FieldKind::Readonly)
            || !field_enabled(&session.resource_id, field.key, &session.values);
        if matches!(field.kind, FieldKind::Repeat) {
            let n = session.repeat.get(field.key).map_or(0, Vec::len);
            for index in 0..n {
                rows.push(FormRow::RepeatItem {
                    locked,
                    field,
                    index,
                });
            }
            rows.push(FormRow::RepeatAdd { locked, field });
        } else {
            rows.push(FormRow::Field { locked, field });
        }
    }
    rows
}

fn repeat_from_schema(
    schema: &FormSchema,
    values: &HashMap<String, String>,
) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    for section in schema.sections.iter().chain(schema.create_sections.iter()) {
        for field in section.fields {
            if matches!(field.kind, FieldKind::Repeat) {
                let raw = values.get(field.key).map_or("", String::as_str);
                out.insert(field.key.to_string(), split_ros_list(raw));
            }
        }
    }
    out
}

fn optional_from_schema(schema: &FormSchema, values: &HashMap<String, String>) -> HashSet<String> {
    schema
        .sections
        .iter()
        .chain(schema.create_sections.iter())
        .flat_map(|section| section.fields)
        .filter_map(|field| {
            let (_, unset, _) = field.kind.optional()?;
            let value = values.get(field.key)?;
            (!value.is_empty() && value != unset).then(|| field.key.to_string())
        })
        .collect()
}

/// Nested picker sitting on a form sheet (live lookup or static enum).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupPicker {
    pub field_key: String,
    pub resource_id: &'static str,
    pub value_key: &'static str,
    pub multiple: bool,
    pub filter: String,
    pub options: Vec<String>,
    /// Wire value to display label for typed enum choices.
    pub labels: HashMap<String, String>,
    pub selected: Vec<String>,
    pub focus: usize,
    pub loading: bool,
    pub error: Option<String>,
    pub request_id: u64,
    pub generation: u64,
}

fn seeded_edit_values(
    resource_id: &str,
    row: &HashMap<String, String>,
    schema: &FormSchema,
) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut values = row.clone();
    let mut original = row.clone();
    for field in schema
        .sections
        .iter()
        .chain(schema.create_sections.iter())
        .flat_map(|section| section.fields)
    {
        if !field_visible(resource_id, field.key, &values) {
            continue;
        }
        let empty = values.get(field.key).is_none_or(String::is_empty);
        if !empty {
            continue;
        }
        let default = default_writable_value(field.kind);
        if default.is_empty() {
            continue;
        }
        values.insert(field.key.to_string(), default.clone());
        original.insert(field.key.to_string(), default);
    }
    (values, original)
}

impl FormSession {
    #[must_use]
    pub fn edit(
        resource_id: impl Into<String>,
        record_id: impl Into<String>,
        row: &HashMap<String, String>,
        schema: &FormSchema,
    ) -> Self {
        let resource_id = resource_id.into();
        let (values, original) = seeded_edit_values(&resource_id, row, schema);
        let repeat = repeat_from_schema(schema, &values);
        let optional_active = optional_from_schema(schema, &values);
        let mut session = Self {
            resource_id,
            record_id: record_id.into(),
            mode: FormMode::Edit,
            section: 0,
            focus: 0,
            offset: 0,
            values,
            original,
            extras: extra_status_fields(schema, row),
            error: None,
            saving: false,
            confirm_discard: false,
            confirm_save: false,
            save_preview_state: SavePreviewState::default(),
            mutation_request_id: None,
            hydrate_request_id: None,
            prompt_command: None,
            prompt_schema: None,
            lookup: None,
            repeat,
            optional_active,
        };
        session.clamp(schema);
        session
    }

    /// Refresh field values from a live fetch without resetting cursor or pickers.
    pub fn apply_live_row(&mut self, row: &HashMap<String, String>, schema: &FormSchema) {
        let (values, original) = seeded_edit_values(&self.resource_id, row, schema);
        self.values = values;
        self.original = original;
        self.repeat = repeat_from_schema(schema, &self.values);
        self.optional_active = optional_from_schema(schema, &self.values);
        self.extras = extra_status_fields(schema, row);
        self.clamp(schema);
    }

    /// Merge a fuller source record (for example Ethernet print) into an open editor.
    pub fn absorb_record(&mut self, row: &HashMap<String, String>, schema: &FormSchema) {
        for (key, value) in row {
            if key == ".id" {
                continue;
            }
            self.original
                .entry(key.clone())
                .or_insert_with(|| value.clone());
            self.values
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
        for field in schema
            .sections
            .iter()
            .chain(schema.create_sections.iter())
            .flat_map(|section| section.fields)
        {
            if !field_visible(&self.resource_id, field.key, &self.values) {
                continue;
            }
            let empty = self.values.get(field.key).is_none_or(String::is_empty);
            if !empty {
                continue;
            }
            let default = default_writable_value(field.kind);
            if default.is_empty() {
                continue;
            }
            self.values.insert(field.key.to_string(), default.clone());
            self.original.insert(field.key.to_string(), default);
        }
        self.repeat = repeat_from_schema(schema, &self.values);
        self.optional_active = optional_from_schema(schema, &self.values);
        self.extras = extra_status_fields(schema, &self.original);
    }

    #[must_use]
    pub fn create(resource_id: impl Into<String>, schema: &FormSchema) -> Self {
        let mut values = HashMap::new();
        for section in schema.sections_for(true) {
            for field in section.fields {
                values
                    .entry(field.key.to_string())
                    .or_insert_with(|| default_writable_value(field.kind));
            }
        }
        let repeat = repeat_from_schema(schema, &values);
        let optional_active = HashSet::new();
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
            save_preview_state: SavePreviewState::default(),
            mutation_request_id: None,
            hydrate_request_id: None,
            prompt_command: None,
            prompt_schema: None,
            lookup: None,
            repeat,
            optional_active,
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
        let original = values.clone();
        let repeat = repeat_from_schema(schema, &values);
        let optional_active = optional_from_schema(schema, &values);
        Self {
            resource_id: resource_id.into(),
            record_id: record_id.into(),
            mode: FormMode::Create,
            section: 0,
            focus: 0,
            offset: 0,
            values,
            original,
            extras: Vec::new(),
            error: None,
            saving: false,
            confirm_discard: false,
            confirm_save: false,
            save_preview_state: SavePreviewState::default(),
            mutation_request_id: None,
            hydrate_request_id: None,
            prompt_command: Some(command),
            prompt_schema: Some(schema),
            lookup: None,
            repeat,
            optional_active,
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
        let repeat = repeat_from_schema(schema, &values);
        let optional_active = optional_from_schema(schema, &values);
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
            save_preview_state: SavePreviewState::default(),
            mutation_request_id: None,
            hydrate_request_id: None,
            prompt_command: Some(command),
            prompt_schema: Some(schema),
            lookup: None,
            repeat,
            optional_active,
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

    pub fn open_save_preview(&mut self) {
        self.confirm_save = true;
        self.save_preview_state = SavePreviewState::Ready;
        self.error = None;
    }

    pub fn begin_save(&mut self) {
        self.saving = true;
        self.error = None;
        if self.confirm_save {
            self.save_preview_state = SavePreviewState::Pending;
        }
    }

    pub fn close_save_preview(&mut self) {
        self.confirm_save = false;
        self.save_preview_state = SavePreviewState::Ready;
        self.mutation_request_id = None;
        self.error = None;
    }

    pub fn apply_mutation_error(&mut self, error: String) {
        self.saving = false;
        self.mutation_request_id = None;
        self.error = Some(error);
        if self.confirm_save {
            self.save_preview_state = SavePreviewState::Failed;
        }
    }

    pub fn track_mutation_request(&mut self, request_id: u64) {
        if self.saving {
            self.mutation_request_id = Some(request_id);
        }
    }

    #[must_use]
    pub fn accepts_mutation_result(&self, request_id: u64) -> bool {
        self.mutation_request_id == Some(request_id)
    }

    #[must_use]
    pub fn save_preview_pending(&self) -> bool {
        self.confirm_save && self.save_preview_state == SavePreviewState::Pending
    }

    #[must_use]
    pub fn save_preview_failed(&self) -> bool {
        self.confirm_save && self.save_preview_state == SavePreviewState::Failed
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

    /// Unmapped print keys. Switch Port keeps those on the details pane only.
    #[must_use]
    pub fn sheet_extras(&self) -> &[(String, String)] {
        if self.resource_id == "switch-port" {
            &[]
        } else {
            &self.extras
        }
    }

    pub fn clamp(&mut self, schema: &FormSchema) {
        let sections = self.schema_sections(schema);
        if sections.is_empty() {
            self.section = 0;
            self.focus = 0;
            return;
        }
        self.section = 0;
        let len = self.visible_rows(schema).len().max(1);
        self.focus = self.focus.min(len - 1);
        let max_off = len.saturating_sub(1);
        self.offset = self.offset.min(max_off);
        if self.focus < self.offset {
            self.offset = self.focus;
        }
    }

    #[must_use]
    pub fn visible_fields<'a>(&self, schema: &'a FormSchema) -> Vec<(bool, &'a FieldSpec)> {
        self.schema_sections(schema)
            .iter()
            .flat_map(|section| {
                section
                    .fields
                    .iter()
                    .filter(|field| field_visible(&self.resource_id, field.key, &self.values))
                    .map(|field| {
                        (
                            section.read_only
                                || matches!(field.kind, FieldKind::Readonly)
                                || !field_enabled(&self.resource_id, field.key, &self.values),
                            field,
                        )
                    })
            })
            .collect()
    }

    fn visible_rows<'a>(&self, schema: &'a FormSchema) -> Vec<FormRow<'a>> {
        let mut rows = Vec::new();
        for section in self.schema_sections(schema) {
            rows.extend(rows_for_section(self, section));
        }
        rows
    }

    pub fn move_section(&mut self, schema: &FormSchema, delta: isize) {
        let _ = (schema, delta);
        self.section = 0;
    }

    pub fn jump_section(&mut self, schema: &FormSchema, index: usize) {
        let _ = (schema, index);
        self.section = 0;
    }

    pub fn move_field(&mut self, schema: &FormSchema, delta: isize) {
        let len = self.visible_rows(schema).len();
        if len == 0 {
            return;
        }
        self.focus = navigation::moved_index(self.focus, delta, len);
    }

    #[must_use]
    pub fn can_move_field(&self, schema: &FormSchema, delta: isize) -> bool {
        let len = self.visible_rows(schema).len();
        if len == 0 {
            return false;
        }
        navigation::moved_index(self.focus, delta, len) != self.focus
    }

    #[must_use]
    pub fn focused_takes_typed_input(&self, schema: &FormSchema) -> bool {
        match self.visible_rows(schema).get(self.focus).copied() {
            Some(FormRow::RepeatItem { locked, .. }) => !locked,
            Some(FormRow::Field { locked, field }) => {
                !locked
                    && match field.kind.optional() {
                        Some((kind, _, _)) => {
                            self.optional_active.contains(field.key) && kind.takes_typed_input()
                        }
                        None => field.kind.takes_typed_input(),
                    }
            }
            Some(FormRow::RepeatAdd { .. }) | None => false,
        }
    }

    pub fn insert_char(&mut self, schema: &FormSchema, ch: char) {
        if !is_printable_char(ch) {
            return;
        }
        let Some(row) = self.visible_rows(schema).get(self.focus).copied() else {
            return;
        };
        match row {
            FormRow::RepeatItem {
                locked,
                field,
                index,
            } if !locked => {
                let current = self
                    .repeat
                    .get(field.key)
                    .and_then(|items| items.get(index))
                    .map_or("", String::as_str);
                if !field.kind.accepts_char(field.key, current, ch) {
                    return;
                }
                if let Some(item) = self
                    .repeat
                    .entry(field.key.to_string())
                    .or_default()
                    .get_mut(index)
                {
                    item.push(ch);
                }
                self.write_repeat(field.key);
            }
            FormRow::Field { locked, field } if !locked && field.kind.takes_typed_input() => {
                if field.kind.optional().is_some() && !self.optional_active.contains(field.key) {
                    return;
                }
                let current = self.values.get(field.key).map_or("", String::as_str);
                if !field.kind.accepts_char(field.key, current, ch) {
                    return;
                }
                self.values
                    .entry(field.key.to_string())
                    .or_default()
                    .push(ch);
            }
            _ => {}
        }
    }

    pub fn backspace(&mut self, schema: &FormSchema) {
        let Some(row) = self.visible_rows(schema).get(self.focus).copied() else {
            return;
        };
        match row {
            FormRow::RepeatItem {
                locked,
                field,
                index,
            } if !locked => {
                let empty = self
                    .repeat
                    .get(field.key)
                    .and_then(|items| items.get(index))
                    .is_none_or(String::is_empty);
                if empty {
                    if let Some(items) = self.repeat.get_mut(field.key)
                        && index < items.len()
                    {
                        items.remove(index);
                    }
                    self.write_repeat(field.key);
                    self.focus_repeat_after_remove(schema, field.key, index);
                } else if let Some(item) = self
                    .repeat
                    .get_mut(field.key)
                    .and_then(|items| items.get_mut(index))
                {
                    item.pop();
                    self.write_repeat(field.key);
                }
            }
            FormRow::Field { locked, field } if !locked && field.kind.takes_typed_input() => {
                if field.kind.optional().is_some() && !self.optional_active.contains(field.key) {
                    return;
                }
                self.values.entry(field.key.to_string()).or_default().pop();
            }
            _ => {}
        }
    }

    pub fn activate(&mut self, schema: &FormSchema) {
        let Some(row) = self.visible_rows(schema).get(self.focus).copied() else {
            return;
        };
        if row.locked() {
            return;
        }
        match row {
            FormRow::RepeatAdd { field, .. } => {
                self.repeat_push_empty(field.key);
                let index = self.last_repeat_index(field.key);
                self.focus_repeat_item(schema, field.key, index);
            }
            FormRow::RepeatItem { field, index, .. } => {
                let filled = self
                    .repeat
                    .get(field.key)
                    .and_then(|items| items.get(index))
                    .is_some_and(|item| !item.is_empty());
                if filled {
                    self.repeat_push_empty(field.key);
                    let last = self.last_repeat_index(field.key);
                    self.focus_repeat_item(schema, field.key, last);
                }
            }
            FormRow::Field { field, .. } => self.activate_scalar(field),
        }
        self.clamp(schema);
    }

    fn activate_scalar(&mut self, field: &FieldSpec) {
        if let Some((kind, _, _)) = field.kind.optional() {
            if !self.optional_active.contains(field.key) {
                self.optional_active.insert(field.key.to_string());
                self.values.insert(field.key.to_string(), String::new());
                return;
            }
            if let ScalarKind::Enum { choices } = kind {
                self.open_static_picker(field.key, choices);
            }
            return;
        }
        match field.kind {
            FieldKind::Toggle | FieldKind::InvertedToggle => {
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
                let options: Vec<String> =
                    values.iter().map(|value| (*value).to_string()).collect();
                let focus = options
                    .iter()
                    .position(|option| option == &now)
                    .unwrap_or(0);
                let generation = self
                    .lookup
                    .as_ref()
                    .map_or(1, |picker| picker.generation.wrapping_add(1));
                self.lookup = Some(Box::new(LookupPicker {
                    field_key: field.key.to_string(),
                    resource_id: "",
                    value_key: "",
                    multiple: false,
                    filter: String::new(),
                    options,
                    labels: HashMap::new(),
                    selected: Vec::new(),
                    focus,
                    loading: false,
                    error: None,
                    request_id: 0,
                    generation,
                }));
            }
            FieldKind::LabeledEnum { choices } => self.open_static_picker(field.key, choices),
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
                    labels: HashMap::new(),
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

    fn open_static_picker(&mut self, field_key: &str, choices: &[mtui_core::EnumChoice]) {
        let now = self.values.get(field_key).cloned().unwrap_or_default();
        let options: Vec<String> = choices
            .iter()
            .map(|choice| choice.value.to_string())
            .collect();
        let labels = choices
            .iter()
            .map(|choice| (choice.value.to_string(), choice.label.to_string()))
            .collect();
        let focus = options
            .iter()
            .position(|option| option == &now)
            .unwrap_or(0);
        let generation = self
            .lookup
            .as_ref()
            .map_or(1, |picker| picker.generation.wrapping_add(1));
        self.lookup = Some(Box::new(LookupPicker {
            field_key: field_key.to_string(),
            resource_id: "",
            value_key: "",
            multiple: false,
            filter: String::new(),
            options,
            labels,
            selected: Vec::new(),
            focus,
            loading: false,
            error: None,
            request_id: 0,
            generation,
        }));
    }

    pub fn remove_optional(&mut self, schema: &FormSchema) {
        let Some(FormRow::Field {
            locked: false,
            field,
        }) = self.visible_rows(schema).get(self.focus).copied()
        else {
            return;
        };
        let Some((_, unset, _)) = field.kind.optional() else {
            return;
        };
        self.optional_active.remove(field.key);
        if self.mode == FormMode::Create {
            self.values.remove(field.key);
        } else {
            self.values.insert(field.key.to_string(), unset.to_string());
        }
    }

    fn last_repeat_index(&self, key: &str) -> usize {
        self.repeat
            .get(key)
            .map_or(0, |items| items.len().saturating_sub(1))
    }

    fn write_repeat(&mut self, key: &str) {
        let joined = self
            .repeat
            .get(key)
            .map_or_else(String::new, |items| join_ros_list(items));
        self.values.insert(key.to_string(), joined);
    }

    fn repeat_push_empty(&mut self, key: &str) {
        self.repeat
            .entry(key.to_string())
            .or_default()
            .push(String::new());
        self.write_repeat(key);
    }

    fn focus_repeat_item(&mut self, schema: &FormSchema, key: &str, index: usize) {
        if let Some(focus) = self.visible_rows(schema).iter().position(|row| {
            matches!(
                row,
                FormRow::RepeatItem {
                    field,
                    index: item,
                    ..
                } if field.key == key && *item == index
            )
        }) {
            self.focus = focus;
        }
    }

    fn focus_repeat_after_remove(&mut self, schema: &FormSchema, key: &str, index: usize) {
        let rows = self.visible_rows(schema);
        let target = if self.repeat.get(key).is_some_and(|items| !items.is_empty()) {
            let item = index.min(self.last_repeat_index(key));
            rows.iter().position(|row| {
                matches!(
                    row,
                    FormRow::RepeatItem {
                        field,
                        index: found,
                        ..
                    } if field.key == key && *found == item
                )
            })
        } else {
            rows.iter()
                .position(|row| matches!(row, FormRow::RepeatAdd { field, .. } if field.key == key))
        };
        if let Some(focus) = target {
            self.focus = focus;
        }
        self.clamp(schema);
    }

    #[must_use]
    pub fn focused_spec<'a>(&self, schema: &'a FormSchema) -> Option<&'a FieldSpec> {
        self.visible_rows(schema)
            .get(self.focus)
            .map(|row| row.field())
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
        let sheet_id = self.resource_id.clone();
        let picker = self.lookup.as_mut().expect("lookup still open");
        picker.loading = false;
        picker.error = error;
        let options = prepare_lookup_options(&sheet_id, picker.resource_id, options);
        picker.options = if matches!(
            picker.field_key.as_str(),
            "raid-master" | "media-interface" | "crypted-backend"
        ) {
            with_leading_none(options)
        } else {
            options
        };
        let filtered = filtered_picker_options(picker);
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
        let len = filtered_picker_options(picker).len();
        if len == 0 {
            picker.focus = 0;
            return;
        }
        picker.focus = navigation::moved_index(picker.focus, delta, len);
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
        let Some(value) = filtered_picker_options(picker).get(picker.focus).cloned() else {
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
        let filtered = filtered_picker_options(&picker);
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

fn filtered_picker_options(picker: &LookupPicker) -> Vec<String> {
    let q = picker.filter.to_ascii_lowercase();
    picker
        .options
        .iter()
        .filter(|option| {
            q.is_empty()
                || option.to_ascii_lowercase().contains(&q)
                || picker
                    .labels
                    .get(*option)
                    .map_or_else(
                        || enum_display_value(&picker.field_key, option),
                        String::clone,
                    )
                    .to_ascii_lowercase()
                    .contains(&q)
        })
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
    let sections = session.schema_sections(schema);
    let heading_n = sections
        .iter()
        .filter(|section| {
            !rows_for_section(session, section).is_empty()
                || (section.read_only && !session.sheet_extras().is_empty())
        })
        .count();
    let field_n = session
        .visible_rows(schema)
        .len()
        .saturating_add(heading_n)
        .saturating_add(session.sheet_extras().len());
    let chrome = 2u16.saturating_add(SHEET_HINT_ROWS);
    let geometry = layout::sheet_geometry(area.width, area.height, field_n.max(3), chrome);
    let width = geometry.width;
    let height = geometry.height;
    let list_h = geometry.viewport_height;
    let focus_line = sheet_focus_line(session, sections);
    let field_view = ScrollView::around_focus(focus_line, list_h, field_n);
    let rect = compact_modal_rect(area, width, height);
    frame.render_widget(Clear, rect);
    paint_form_panel(
        frame,
        rect,
        session,
        schema,
        sections,
        styles,
        FormChrome::Modal {
            range_label: &field_view.range_label(),
        },
    );
    render_form_nested(frame, area, session, schema, styles);
}

/// Paint the same field rows into a content pane (no dimmed modal chrome).
pub fn render_form_page(
    frame: &mut Frame<'_>,
    area: Rect,
    session: &FormSession,
    schema: &FormSchema,
    styles: &Styles,
) {
    let sections = session.schema_sections(schema);
    paint_form_panel(
        frame,
        area,
        session,
        schema,
        sections,
        styles,
        FormChrome::Page,
    );
}

/// Lookup picker and save preview sit on the full canvas for both modal and page forms.
pub fn render_form_nested(
    frame: &mut Frame<'_>,
    area: Rect,
    session: &FormSession,
    schema: &FormSchema,
    styles: &Styles,
) {
    if let Some(picker) = &session.lookup {
        render_lookup_picker(frame, area, picker, styles);
    }
    if session.confirm_save {
        render_save_preview(frame, area, session, schema, styles);
    }
}

#[derive(Clone, Copy)]
enum FormChrome<'a> {
    Modal { range_label: &'a str },
    Page,
}

fn paint_form_panel(
    frame: &mut Frame<'_>,
    rect: Rect,
    session: &FormSession,
    schema: &FormSchema,
    sections: &[FormSection],
    styles: &Styles,
    chrome: FormChrome<'_>,
) {
    let title = sheet_title(session, schema);
    let (range_label, modal) = match chrome {
        FormChrome::Modal { range_label } => (range_label, true),
        FormChrome::Page => ("", false),
    };
    let border = if session.confirm_discard {
        styles.alert
    } else if modal {
        styles.border
    } else {
        styles.focus
    };
    let mut title_spans = vec![Span::styled(format!(" {title} "), styles.title)];
    if !range_label.is_empty() {
        title_spans.push(Span::styled(format!("{range_label} "), styles.muted));
    }
    let block = Block::default()
        .title(Line::from(title_spans))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border)
        .style(styles.text)
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(SHEET_HINT_ROWS)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(sheet_field_lines(
            session,
            schema,
            sections,
            usize::from(chunks[0].width.max(1)),
            usize::from(chunks[0].height.max(1)),
            styles,
        )),
        chunks[0],
    );

    let hint = sheet_hint(session, schema);
    let hint_style = if session.error.is_some() || session.confirm_discard {
        styles.alert
    } else {
        styles.muted
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            clip_line(&hint, usize::from(chunks[1].width.max(1))),
            hint_style,
        ))),
        chunks[1],
    );
}

fn sheet_field_lines(
    session: &FormSession,
    _schema: &FormSchema,
    sections: &[FormSection],
    width: usize,
    height: usize,
    styles: &Styles,
) -> Vec<Line<'static>> {
    enum DisplayLine<'a> {
        Heading(&'a str),
        Row(usize, FormRow<'a>),
        Extra(&'a str, &'a str),
    }

    let mut display = Vec::new();
    let mut row_index = 0usize;
    let mut extras_rendered = false;
    for section in sections {
        let rows = rows_for_section(session, section);
        let extras = session.sheet_extras();
        let has_extras = section.read_only && !extras.is_empty();
        if rows.is_empty() && !has_extras {
            continue;
        }
        display.push(DisplayLine::Heading(section.label));
        for row in rows {
            display.push(DisplayLine::Row(row_index, row));
            row_index += 1;
        }
        if has_extras {
            for (key, value) in extras {
                display.push(DisplayLine::Extra(key, value));
            }
            extras_rendered = true;
        }
    }
    if !extras_rendered && !session.sheet_extras().is_empty() {
        display.push(DisplayLine::Heading("Status"));
        for (key, value) in session.sheet_extras() {
            display.push(DisplayLine::Extra(key, value));
        }
    }

    let focus_line = display
        .iter()
        .position(|line| matches!(line, DisplayLine::Row(index, _) if *index == session.focus))
        .unwrap_or(0);
    let view = ScrollView::around_focus(focus_line, height.max(1), display.len());
    display
        .into_iter()
        .skip(view.offset)
        .take(view.visible)
        .enumerate()
        .map(|(window_i, line)| {
            let gutter = view.gutter(window_i);
            match line {
                DisplayLine::Heading(label) => section_heading_line(label, width, gutter, styles),
                DisplayLine::Row(index, row) => {
                    rows::row_line(session, row, index == session.focus, width, gutter, styles)
                }
                DisplayLine::Extra(key, value) => {
                    let mut line = Line::from(vec![
                        Span::styled(format!("  {key:<22} "), styles.muted),
                        Span::styled(value.to_string(), styles.text),
                    ]);
                    if gutter != ' ' {
                        line.spans
                            .push(Span::styled(gutter.to_string(), styles.quiet));
                    }
                    line
                }
            }
        })
        .collect()
}

fn sheet_focus_line(session: &FormSession, sections: &[FormSection]) -> usize {
    let mut line = 0usize;
    let mut row_index = 0usize;
    for section in sections {
        let rows = rows_for_section(session, section);
        let extras = session.sheet_extras();
        let has_extras = section.read_only && !extras.is_empty();
        if rows.is_empty() && !has_extras {
            continue;
        }
        line += 1;
        for _ in rows {
            if row_index == session.focus {
                return line;
            }
            row_index += 1;
            line += 1;
        }
        if has_extras {
            line += extras.len();
        }
    }
    0
}

fn section_heading_line(label: &str, width: usize, gutter: char, styles: &Styles) -> Line<'static> {
    let gutter_width = usize::from(gutter != ' ');
    let body = format!("── {label} ");
    let fill = "─".repeat(
        width
            .saturating_sub(body.width())
            .saturating_sub(gutter_width),
    );
    let mut spans = vec![Span::styled(format!("{body}{fill}"), styles.key)];
    if gutter != ' ' {
        spans.push(Span::styled(gutter.to_string(), styles.quiet));
    }
    Line::from(spans)
}

fn sheet_hint(session: &FormSession, schema: &FormSchema) -> String {
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
    let field_hint = session
        .visible_rows(schema)
        .get(session.focus)
        .copied()
        .map_or("tab field", |row| {
            if row.locked() {
                FieldKind::Readonly.edit_hint()
            } else {
                match row {
                    FormRow::RepeatAdd { .. } => "enter add",
                    FormRow::RepeatItem { .. } => "type value   enter add   bksp empty removes",
                    FormRow::Field { field, .. } => field.kind.edit_hint(),
                }
            }
        });
    format!("↑↓ / tab field   {field_hint}   ctrl+s save   esc")
}

fn render_lookup_picker(frame: &mut Frame<'_>, area: Rect, picker: &LookupPicker, styles: &Styles) {
    dim_canvas(frame, area, styles);
    let width = area.width.saturating_sub(8).clamp(20, 52);
    let height = area.height.saturating_sub(4).clamp(5, 16);
    let rect = compact_modal_rect(area, width, height);
    frame.render_widget(Clear, rect);

    let title = if picker.resource_id.is_empty() {
        " Select "
    } else if picker.multiple {
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
    let filtered = filtered_picker_options(picker);
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
        let label = picker
            .labels
            .get(option)
            .cloned()
            .unwrap_or_else(|| enum_display_value(&picker.field_key, option));
        let body = format!("{caret} {mark}{label}");
        let style = if idx == picker.focus {
            styles.focus
        } else {
            styles.text
        };
        lines.push(Line::from(Span::styled(clip_line(&body, width), style)));
    }
    lines
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

fn enum_display_value(key: &str, raw: &str) -> String {
    match (key, raw) {
        ("remote-log-format", "syslog") | ("syslog-time-format", "bsd-syslog") => {
            "BSD syslog".into()
        }
        ("remote-log-format", "cef") => "CEF".into(),
        ("remote-protocol", "tls") => "TLS".into(),
        ("syslog-time-format", "iso8601") => "ISO 8601".into(),
        ("version", "ipfix") => "IPFIX".into(),
        _ => raw.to_string(),
    }
}

#[cfg(any())]
mod retired_row_rendering {
    use super::*;

    fn row_line(
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
            let gutter_style = if gutter == '▐' {
                styles.key
            } else {
                styles.quiet
            };
            spans.push(Span::styled(gutter.to_string(), gutter_style));
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
        let body = pad_visual("+ add", width);
        vec![Span::styled(body, style)]
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
        if let FieldKind::Optional {
            kind, unset_label, ..
        } = field.kind
        {
            if !optional_active {
                let body = format!("+ set ({unset_label})");
                return vec![Span::styled(
                    pad_visual(&body, width),
                    if focused && !locked {
                        styles.focus
                    } else {
                        styles.muted
                    },
                )];
            }
            return scalar_control(
                kind,
                field.key,
                raw,
                locked,
                focused,
                width,
                chrome,
                value_style,
            );
        }
        match field.kind {
            FieldKind::Toggle | FieldKind::InvertedToggle => {
                toggle_control(field.kind.toggle_is_on(raw), locked, focused, width, styles)
            }
            FieldKind::Enum { .. } | FieldKind::LabeledEnum { .. } | FieldKind::Lookup { .. } => {
                let shown = if raw.is_empty() {
                    "—".to_string()
                } else {
                    let typed = field.kind.display_value(raw);
                    if typed == raw {
                        enum_display_value(field.key, raw)
                    } else {
                        typed
                    }
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
        } else if shown.is_empty() && focused {
            String::new()
        } else {
            shown
        };
        let (open, trail, close) = if matches!(kind, ScalarKind::Enum { .. }) {
            ('<', '▾', '>')
        } else {
            ('[', ' ', ']')
        };
        let mut spans = vec![Span::styled("− ".to_string(), chrome)];
        spans.extend(slot_control(
            &if shown == raw {
                enum_display_value(field_key, &shown)
            } else {
                shown
            },
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

    fn rendered_text(terminal: &Terminal<TestBackend>) -> String {
        let buffer = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                rendered.push_str(buffer[(x, y)].symbol());
            }
            rendered.push('\n');
        }
        rendered
    }

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
    fn ethernet_sheet_omits_poe_and_sfp_without_port_capabilities() {
        let schema = mtui_core::resource_by_id("ethernet")
            .and_then(|spec| spec.form)
            .expect("ethernet form");
        let row = HashMap::from([("name".into(), "ether1".into())]);
        let session = FormSession::edit("ethernet", "*1", &row, schema);
        assert!(!session.values.contains_key("poe-out"));
        assert!(!session.values.contains_key("sfp-rate-select"));
        let keys: Vec<_> = session
            .visible_fields(schema)
            .into_iter()
            .map(|(_, field)| field.key)
            .collect();
        assert!(!keys.contains(&"poe-out"));
        assert!(!keys.contains(&"sfp-rate-select"));
        assert!(keys.contains(&"name"));
        assert!(keys.contains(&"auto-negotiation"));
    }

    #[test]
    fn ethernet_sheet_ignores_sfp_name_without_sfp_print_attrs() {
        let schema = mtui_core::resource_by_id("ethernet")
            .and_then(|spec| spec.form)
            .expect("ethernet form");
        let row = HashMap::from([
            ("name".into(), "sfp1".into()),
            ("default-name".into(), "sfp1".into()),
        ]);
        let session = FormSession::edit("ethernet", "*1", &row, schema);
        let keys: Vec<_> = session
            .visible_fields(schema)
            .into_iter()
            .map(|(_, field)| field.key)
            .collect();
        assert!(!keys.contains(&"sfp-rate-select"));
        assert!(!keys.contains(&"poe-out"));
    }

    #[test]
    fn ethernet_sheet_absorbs_poe_attributes_from_source_record() {
        let schema = mtui_core::resource_by_id("ethernet")
            .and_then(|spec| spec.form)
            .expect("ethernet form");
        let row = HashMap::from([("name".into(), "ether3".into())]);
        let mut session = FormSession::edit("ethernet", "*1", &row, schema);
        session.absorb_record(
            &HashMap::from([("poe-out".into(), "auto-on".into())]),
            schema,
        );
        let keys: Vec<_> = session
            .visible_fields(schema)
            .into_iter()
            .map(|(_, field)| field.key)
            .collect();
        assert!(keys.contains(&"poe-out"));
    }

    #[test]
    fn ethernet_sheet_absorbs_sfp_print_attributes() {
        let schema = mtui_core::resource_by_id("ethernet")
            .and_then(|spec| spec.form)
            .expect("ethernet form");
        let row = HashMap::from([
            ("name".into(), "uplink".into()),
            ("default-name".into(), "sfp1".into()),
        ]);
        let mut session = FormSession::edit("ethernet", "*1", &row, schema);
        session.absorb_record(
            &HashMap::from([
                ("sfp-shutdown-temperature".into(), "95C".into()),
                ("sfp-ignore-rx-los".into(), "no".into()),
            ]),
            schema,
        );
        let keys: Vec<_> = session
            .visible_fields(schema)
            .into_iter()
            .map(|(_, field)| field.key)
            .collect();
        assert!(keys.contains(&"sfp-rate-select"));
        assert!(keys.contains(&"sfp-ignore-rx-los"));
        assert!(!keys.contains(&"poe-out"));
    }

    #[test]
    fn ethernet_sheet_shows_gated_sections_when_capabilities_are_set() {
        let schema = mtui_core::resource_by_id("ethernet")
            .and_then(|spec| spec.form)
            .expect("ethernet form");
        let row = HashMap::from([
            ("name".into(), "ether5".into()),
            ("poe-out".into(), "auto-on".into()),
            ("sfp-shutdown-temperature".into(), "95C".into()),
        ]);
        let session = FormSession::edit("ethernet", "*1", &row, schema);
        let keys: Vec<_> = session
            .visible_fields(schema)
            .into_iter()
            .map(|(_, field)| field.key)
            .collect();
        assert!(keys.contains(&"poe-out"));
        assert!(keys.contains(&"sfp-rate-select"));
    }

    #[test]
    fn field_movement_crosses_sections_and_clamps_without_wrapping() {
        let schema = sample_schema();
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        let mut session = FormSession::edit("interfaces", "*1", &row, &schema);
        session.move_field(&schema, -1);
        assert_eq!(session.focus, 0);
        session.move_field(&schema, 1);
        assert_eq!(session.focus, 1);
        session.move_field(&schema, 1);
        assert_eq!(session.focus, 2);
        session.move_field(&schema, 1);
        assert_eq!(session.focus, 2);
        assert!(!session.can_move_field(&schema, 1));
        assert!(session.can_move_field(&schema, -1));

        session.move_section(&schema, -1);
        assert_eq!(session.section, 0);
        session.move_section(&schema, 1);
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
    fn repeat_field_adds_types_and_removes_rows() {
        let schema = FormSchema {
            title_key: "broadcast-addresses",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "broadcast",
                        label: "Broadcast",
                        kind: FieldKind::Toggle,
                    },
                    FieldSpec {
                        key: "broadcast-addresses",
                        label: "Broadcast Addresses",
                        kind: FieldKind::Repeat,
                    },
                ],
            }],
            create_sections: &[],
        };
        let mut row = HashMap::new();
        row.insert("broadcast".into(), "true".into());
        row.insert("broadcast-addresses".into(), "10.0.0.255,10.0.1.255".into());
        let mut session = FormSession::edit("ntp-server", "*0", &row, &schema);
        assert_eq!(session.visible_rows(&schema).len(), 4);

        session.focus = 3;
        session.activate(&schema);
        session.insert_char(&schema, '9');
        assert_eq!(
            session
                .values
                .get("broadcast-addresses")
                .map(String::as_str),
            Some("10.0.0.255,10.0.1.255,9")
        );

        session.backspace(&schema);
        session.backspace(&schema);
        assert_eq!(
            session
                .values
                .get("broadcast-addresses")
                .map(String::as_str),
            Some("10.0.0.255,10.0.1.255")
        );
        assert_eq!(session.visible_rows(&schema).len(), 4);
        assert!(session.focused_takes_typed_input(&schema));
        session.focus = 3;
        assert!(!session.focused_takes_typed_input(&schema));
    }

    #[test]
    fn optional_number_starts_collapsed_and_can_be_added_and_removed() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[FieldSpec {
                    key: "mtu",
                    label: "MTU",
                    kind: FieldKind::Optional {
                        kind: ScalarKind::Number {
                            min: Some(68),
                            max: Some(65_535),
                        },
                        unset: "auto",
                        unset_label: "automatic",
                    },
                }],
            }],
            create_sections: &[],
        };
        let row = HashMap::from([("mtu".into(), "auto".into())]);
        let mut session = FormSession::edit("interfaces", "*1", &row, &schema);
        assert!(!session.optional_active.contains("mtu"));
        assert!(!session.focused_takes_typed_input(&schema));
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_form_sheet(frame, frame.area(), &session, &schema, &styles))
            .expect("draw");
        let rendered = rendered_text(&terminal);
        assert!(rendered.contains("+ set (automatic)"));
        assert!(!rendered.contains("[auto"));

        session.activate(&schema);
        assert!(session.optional_active.contains("mtu"));
        assert_eq!(session.values.get("mtu").map(String::as_str), Some(""));
        session.insert_char(&schema, '1');
        session.insert_char(&schema, '5');
        session.insert_char(&schema, '0');
        session.insert_char(&schema, '0');
        assert_eq!(session.values.get("mtu").map(String::as_str), Some("1500"));
        session.remove_optional(&schema);
        assert!(!session.optional_active.contains("mtu"));
        assert_eq!(session.values.get("mtu").map(String::as_str), Some("auto"));
    }

    #[test]
    fn labeled_enum_picker_commits_wire_value() {
        const CHOICES: &[mtui_core::EnumChoice] = &[
            mtui_core::EnumChoice {
                label: "Automatic",
                value: "auto",
            },
            mtui_core::EnumChoice {
                label: "One gigabit",
                value: "1G-baseT-full",
            },
        ];
        let schema = FormSchema {
            title_key: "speed",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[FieldSpec {
                    key: "speed",
                    label: "Speed",
                    kind: FieldKind::LabeledEnum { choices: CHOICES },
                }],
            }],
            create_sections: &[],
        };
        let mut session = FormSession::create("ethernet", &schema);
        session.activate(&schema);
        let picker = session.lookup.as_ref().expect("picker");
        assert_eq!(
            picker.labels.get("1G-baseT-full").map(String::as_str),
            Some("One gigabit")
        );
        session.lookup_move(1);
        session.lookup_confirm();
        assert_eq!(
            session.values.get("speed").map(String::as_str),
            Some("1G-baseT-full")
        );
    }

    #[test]
    fn lte_apn_hides_password_until_authentication_is_set() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "authentication",
                        label: "Authentication",
                        kind: FieldKind::Enum {
                            values: &["none", "pap", "chap"],
                        },
                    },
                    FieldSpec {
                        key: "password",
                        label: "Password",
                        kind: FieldKind::Secret,
                    },
                    FieldSpec {
                        key: "network-mode",
                        label: "Network Mode",
                        kind: FieldKind::Repeat,
                    },
                ],
            }],
            create_sections: &[],
        };
        let mut row = HashMap::new();
        row.insert("authentication".into(), "none".into());
        row.insert("network-mode".into(), "lte".into());
        let mut session = FormSession::edit("lte-apn", "*1", &row, &schema);
        let keys: Vec<_> = session
            .visible_fields(&schema)
            .iter()
            .map(|(_, field)| field.key)
            .collect();
        assert_eq!(keys, ["authentication", "network-mode"]);

        session.activate(&schema);
        let picker = session.lookup.as_ref().expect("select");
        assert_eq!(picker.resource_id, "");
        assert!(!picker.loading);
        assert_eq!(picker.options, ["none", "pap", "chap"]);
        session.lookup_move(2);
        session.lookup_confirm();
        assert_eq!(
            session.values.get("authentication").map(String::as_str),
            Some("chap")
        );
        let keys: Vec<_> = session
            .visible_fields(&schema)
            .iter()
            .map(|(_, field)| field.key)
            .collect();
        assert_eq!(keys, ["authentication", "password", "network-mode"]);
    }

    #[test]
    fn form_sheet_scrolls_repeat_rows_and_pins_hints() {
        let schema = FormSchema {
            title_key: "addrs",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[FieldSpec {
                    key: "addrs",
                    label: "Addresses",
                    kind: FieldKind::Repeat,
                }],
            }],
            create_sections: &[],
        };
        let list = (0..20)
            .map(|i| format!("198.51.100.{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let mut row = HashMap::new();
        row.insert("addrs".into(), list);
        let mut session = FormSession::edit("interfaces", "*1", &row, &schema);
        let tail = session.visible_rows(&schema).len().saturating_sub(1);
        session.focus = tail;
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(64, 14);
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
        assert!(
            rendered.contains("198.51.100.19") || rendered.contains("+ add"),
            "focused tail missing: {rendered}"
        );
        assert!(
            !rendered.contains("[198.51.100.0]"),
            "scrolled sheet still showed the first address: {rendered}"
        );
        assert!(
            rendered.contains("ctrl+s") || rendered.contains("enter add"),
            "hint must stay pinned: {rendered}"
        );
        assert!(
            rendered.contains("/21") || rendered.contains('▐') || rendered.contains('│'),
            "missing scroll chrome: {rendered}"
        );
    }

    #[test]
    fn number_fields_ignore_letters_and_sixth_port_digit() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "remote-port",
                        label: "Remote Port",
                        kind: FieldKind::Number,
                    },
                    FieldSpec {
                        key: "memory-lines",
                        label: "Memory Lines",
                        kind: FieldKind::Number,
                    },
                ],
            }],
            create_sections: &[],
        };
        let mut session = FormSession::create("radius", &schema);
        session.focus = 0;
        session.insert_char(&schema, 'a');
        session.insert_char(&schema, '-');
        for ch in ['6', '5', '5', '3', '5', '9'] {
            session.insert_char(&schema, ch);
        }
        assert_eq!(
            session.values.get("remote-port").map(String::as_str),
            Some("65535")
        );
        session.focus = 1;
        session.insert_char(&schema, 'x');
        session.insert_char(&schema, '1');
        session.insert_char(&schema, '2');
        assert_eq!(
            session.values.get("memory-lines").map(String::as_str),
            Some("12")
        );
    }

    #[test]
    fn empty_enum_defaults_to_first_value_without_dirtying() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &["remote-protocol"],
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
                        key: "remote-protocol",
                        label: "Protocol",
                        kind: FieldKind::Enum {
                            values: &["udp", "tcp"],
                        },
                    },
                ],
            }],
            create_sections: &[],
        };
        let mut row = HashMap::new();
        row.insert("name".into(), "remote".into());
        let session = FormSession::edit("fixture", "*1", &row, &schema);
        assert_eq!(
            session.values.get("remote-protocol").map(String::as_str),
            Some("udp")
        );
        assert!(!session.is_dirty());
    }

    #[test]
    fn empty_ntp_auth_key_defaults_to_none_without_dirtying() {
        let schema = FormSchema {
            title_key: "enabled",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[FieldSpec {
                    key: "auth-key",
                    label: "Auth. Key",
                    kind: FieldKind::Lookup {
                        resource_id: "ntp-keys",
                        value_key: "key-id",
                        multiple: false,
                    },
                }],
            }],
            create_sections: &[],
        };
        let mut session = FormSession::edit("ntp-server", "", &HashMap::new(), &schema);
        assert_eq!(
            session.values.get("auth-key").map(String::as_str),
            Some("none")
        );
        assert!(!session.is_dirty());
        session.activate(&schema);
        let (request_id, generation) = session
            .lookup
            .as_ref()
            .map(|picker| (picker.request_id, picker.generation))
            .expect("picker");
        assert!(session.apply_lookup_result(request_id, generation, vec!["1".into()], None));
        assert_eq!(
            session.lookup.as_ref().unwrap().options,
            vec!["none".to_string(), "1".into()]
        );
    }

    #[test]
    fn romon_port_interface_lookup_prepends_all() {
        let schema = FormSchema {
            title_key: "interface",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[FieldSpec {
                    key: "interface",
                    label: "Interface",
                    kind: FieldKind::Lookup {
                        resource_id: "interfaces",
                        value_key: "name",
                        multiple: false,
                    },
                }],
            }],
            create_sections: &[],
        };
        let mut session = FormSession::create("romon-ports", &schema);
        session.activate(&schema);
        let (request_id, generation) = session
            .lookup
            .as_ref()
            .map(|picker| (picker.request_id, picker.generation))
            .expect("picker");
        assert!(session.apply_lookup_result(
            request_id,
            generation,
            vec!["ether1".into(), "ether2".into()],
            None
        ));
        assert_eq!(
            session.lookup.as_ref().unwrap().options,
            vec!["all".to_string(), "ether1".into(), "ether2".into()]
        );
        session.close_lookup();
        let stale =
            session.apply_lookup_result(request_id, generation, vec!["ether9".into()], None);
        assert!(!stale);
    }

    #[test]
    fn logging_action_shows_fields_for_selected_type() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "target",
                        label: "Type",
                        kind: FieldKind::Enum {
                            values: &["memory", "remote"],
                        },
                    },
                    FieldSpec {
                        key: "memory-lines",
                        label: "Memory Lines",
                        kind: FieldKind::Number,
                    },
                    FieldSpec {
                        key: "remote-log-format",
                        label: "Remote Log Format",
                        kind: FieldKind::Enum {
                            values: &["default", "syslog", "cef"],
                        },
                    },
                    FieldSpec {
                        key: "syslog-facility",
                        label: "Syslog Facility",
                        kind: FieldKind::Enum {
                            values: &["daemon", "kern"],
                        },
                    },
                    FieldSpec {
                        key: "remote",
                        label: "Remote Address",
                        kind: FieldKind::Text,
                    },
                ],
            }],
            create_sections: &[],
        };
        let mut session = FormSession::create("logging-actions", &schema);
        let keys = |session: &FormSession, schema: &FormSchema| {
            session
                .visible_fields(schema)
                .into_iter()
                .map(|(_, field)| field.key)
                .collect::<Vec<_>>()
        };
        assert_eq!(keys(&session, &schema), ["target", "memory-lines"]);
        assert!(
            session
                .visible_fields(&schema)
                .iter()
                .all(|(locked, _)| !*locked)
        );

        session.values.insert("target".into(), "remote".into());
        session
            .values
            .insert("remote-log-format".into(), "default".into());
        assert_eq!(
            keys(&session, &schema),
            ["target", "remote-log-format", "remote"]
        );

        session
            .values
            .insert("remote-log-format".into(), "syslog".into());
        session
            .values
            .insert("syslog-facility".into(), "daemon".into());
        assert_eq!(
            keys(&session, &schema),
            ["target", "remote-log-format", "syslog-facility", "remote"]
        );
        session.focus = 2;
        session.activate(&schema);
        assert!(session.lookup_open());
        session.lookup_move(1);
        session.lookup_confirm();
        assert_eq!(
            session.values.get("syslog-facility").map(String::as_str),
            Some("kern")
        );
        session.focus = 3;
        session.insert_char(&schema, 'x');
        assert_eq!(session.values.get("remote").map(String::as_str), Some("x"));

        session.values.insert("target".into(), "memory".into());
        session.clamp(&schema);
        assert_eq!(keys(&session, &schema), ["target", "memory-lines"]);
        session.focus = 1;
        session.insert_char(&schema, '8');
        assert_eq!(
            session.values.get("memory-lines").map(String::as_str),
            Some("8")
        );
        assert_eq!(session.values.get("remote").map(String::as_str), Some("x"));
    }

    #[test]
    fn traffic_flow_hides_sampling_until_enabled() {
        let schema = FormSchema {
            title_key: "enabled",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "packet-sampling",
                        label: "Packet Sampling",
                        kind: FieldKind::Toggle,
                    },
                    FieldSpec {
                        key: "sampling-interval",
                        label: "Sampling Interval",
                        kind: FieldKind::Number,
                    },
                    FieldSpec {
                        key: "sampling-space",
                        label: "Sampling Space",
                        kind: FieldKind::Number,
                    },
                    FieldSpec {
                        key: "interfaces",
                        label: "Interfaces",
                        kind: FieldKind::Repeat,
                    },
                ],
            }],
            create_sections: &[],
        };
        let mut row = HashMap::new();
        row.insert("packet-sampling".into(), "false".into());
        row.insert("interfaces".into(), "all".into());
        let mut session = FormSession::edit("traffic-flow", "", &row, &schema);
        let keys = |session: &FormSession, schema: &FormSchema| {
            session
                .visible_fields(schema)
                .into_iter()
                .map(|(_, field)| field.key)
                .collect::<Vec<_>>()
        };
        assert_eq!(keys(&session, &schema), ["packet-sampling", "interfaces"]);
        session
            .values
            .insert("packet-sampling".into(), "true".into());
        assert_eq!(
            keys(&session, &schema),
            [
                "packet-sampling",
                "sampling-interval",
                "sampling-space",
                "interfaces"
            ]
        );
        session.focus = 1;
        session.insert_char(&schema, '2');
        assert_eq!(
            session.values.get("sampling-interval").map(String::as_str),
            Some("2")
        );
        session.insert_char(&schema, 'x');
        assert_eq!(
            session.values.get("sampling-interval").map(String::as_str),
            Some("2")
        );
    }

    #[test]
    fn traffic_flow_target_opens_version_select_and_caps_port() {
        let target_schema = FormSchema {
            title_key: "dst-address",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[
                    FieldSpec {
                        key: "version",
                        label: "Version",
                        kind: FieldKind::Enum {
                            values: &["1", "5", "9", "ipfix"],
                        },
                    },
                    FieldSpec {
                        key: "v9-template-refresh",
                        label: "v9 Template Refresh",
                        kind: FieldKind::Number,
                    },
                    FieldSpec {
                        key: "port",
                        label: "Port",
                        kind: FieldKind::Number,
                    },
                ],
            }],
            create_sections: &[],
        };
        let mut target = HashMap::new();
        target.insert("version".into(), "5".into());
        target.insert("port".into(), "2055".into());
        let mut session =
            FormSession::edit("traffic-flow-targets", "*tf1", &target, &target_schema);
        let keys = |session: &FormSession, schema: &FormSchema| {
            session
                .visible_fields(schema)
                .into_iter()
                .map(|(_, field)| field.key)
                .collect::<Vec<_>>()
        };
        assert_eq!(keys(&session, &target_schema), ["version", "port"]);
        session.values.insert("version".into(), "ipfix".into());
        assert_eq!(
            keys(&session, &target_schema),
            ["version", "v9-template-refresh", "port"]
        );
        session.activate(&target_schema);
        assert!(session.lookup_open());
        session.lookup_insert_char('i');
        session.lookup_insert_char('p');
        session.lookup_confirm();
        assert_eq!(
            session.values.get("version").map(String::as_str),
            Some("ipfix")
        );
        session.focus = 2;
        session.insert_char(&target_schema, '9');
        assert_eq!(
            session.values.get("port").map(String::as_str),
            Some("20559")
        );
        session.insert_char(&target_schema, '6');
        assert_eq!(
            session.values.get("port").map(String::as_str),
            Some("20559")
        );
    }

    #[test]
    fn enum_picker_opens_selects_and_filters_display_names() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[FieldSpec {
                    key: "remote-log-format",
                    label: "Remote Log Format",
                    kind: FieldKind::Enum {
                        values: &["default", "syslog", "cef"],
                    },
                }],
            }],
            create_sections: &[],
        };
        let mut session = FormSession::create("interfaces", &schema);
        session.activate(&schema);
        let picker = session.lookup.as_ref().expect("picker");
        assert!(!picker.loading);
        assert!(picker.resource_id.is_empty());
        assert_eq!(picker.options, ["default", "syslog", "cef"]);
        assert_eq!(picker.focus, 0);

        session.lookup_insert_char('b');
        session.lookup_insert_char('s');
        session.lookup_insert_char('d');
        let picker = session.lookup.as_ref().expect("picker");
        assert_eq!(picker.focus, 0);
        session.lookup_confirm();
        assert!(session.lookup.is_none());
        assert_eq!(
            session.values.get("remote-log-format").map(String::as_str),
            Some("syslog")
        );
    }

    #[test]
    fn enum_picker_keeps_a_scrollable_viewport() {
        const OPTIONS: &[&str] = &[
            "opt00", "opt01", "opt02", "opt03", "opt04", "opt05", "opt06", "opt07", "opt08",
            "opt09", "opt10", "opt11", "opt12", "opt13", "opt14", "opt15", "opt16", "opt17",
            "opt18", "opt19",
        ];
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[FieldSpec {
                    key: "choice",
                    label: "Choice",
                    kind: FieldKind::Enum { values: OPTIONS },
                }],
            }],
            create_sections: &[],
        };
        let mut session = FormSession::create("interfaces", &schema);
        session.activate(&schema);
        for _ in 0..19 {
            session.lookup_move(1);
        }
        assert_eq!(session.lookup.as_ref().expect("picker").focus, 19);

        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(36, 10);
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
        assert!(rendered.contains("Select"));
        assert!(rendered.contains("opt19"));
    }

    #[test]
    fn create_enum_defaults_protocol_to_first_value() {
        let schema = FormSchema {
            title_key: "name",
            subtitle_keys: &[],
            sections: &[],
            create_sections: &[FormSection {
                id: "general",
                label: "General",
                read_only: false,
                fields: &[FieldSpec {
                    key: "remote-protocol",
                    label: "Protocol",
                    kind: FieldKind::Enum {
                        values: &["udp", "tcp"],
                    },
                }],
            }],
        };
        let session = FormSession::create("logging-actions", &schema);
        assert_eq!(
            session.values.get("remote-protocol").map(String::as_str),
            Some("udp")
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
        assert!(rendered.contains("── General"));
        assert!(rendered.contains("── Status"));
        assert!(rendered.contains("Name"));
        assert!(rendered.contains("text"));
        assert!(rendered.contains('['));
        assert!(rendered.contains("toggle"));
        assert!(!rendered.contains("1-9 jump"));
        assert!(!rendered.contains("[1 General]"));
    }

    #[test]
    fn switch_port_form_omits_status_extras() {
        let schema = mtui_core::resource_by_id("switch-port")
            .and_then(|spec| spec.form)
            .expect("switch-port form");
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        row.insert("switch".into(), "switch1".into());
        row.insert("invalid".into(), "true".into());
        let session = FormSession::edit("switch-port", "*1", &row, schema);
        assert!(!session.extras.is_empty());
        assert!(session.sheet_extras().is_empty());
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, schema, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("ether1"), "{rendered}");
        assert!(!rendered.contains("── Status"), "{rendered}");
        assert!(!rendered.contains("invalid"), "{rendered}");
    }

    #[test]
    fn navigation_hint_has_no_tab_switching() {
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
        assert!(!rendered.contains("1-9 jump"));
        assert!(!rendered.contains("[ / ] tabs"));
        assert!(rendered.contains("space toggle"));
    }

    #[test]
    fn sheet_title_fits_narrow_labels() {
        let title = "ether1 · ethernet · RUN";
        assert!(title.width() < 40);
    }

    #[test]
    fn form_sheet_renders_on_narrow_short_terminal() {
        let schema = sample_schema();
        let row = HashMap::from([
            ("name".into(), "ether1".into()),
            ("running".into(), "true".into()),
        ]);
        let session = FormSession::edit("interfaces", "*1", &row, &schema);
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(20, 6);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_form_sheet(frame, frame.area(), &session, &schema, &styles))
            .expect("narrow render");
        assert_eq!(terminal.backend().buffer().area.width, 20);
        assert_eq!(terminal.backend().buffer().area.height, 6);
    }

    #[test]
    fn all_section_headings_render_in_schema_order() {
        let schema = sample_schema();
        let mut row = HashMap::new();
        row.insert("name".into(), "ether1".into());
        row.insert("running".into(), "true".into());
        let session = FormSession::edit("interfaces", "*1", &row, &schema);
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render_form_sheet(frame, frame.area(), &session, &schema, &styles))
            .expect("draw");
        let rendered = rendered_text(&terminal);
        let general = rendered.find("── General").expect("general heading");
        let status = rendered.find("── Status").expect("status heading");
        assert!(general < status);
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
    fn ospf_interface_status_sheet_hides_tabs_and_pins_hints() {
        let spec = mtui_core::resource_by_id("ospf-interfaces").expect("ospf-interfaces");
        let schema = spec.form.expect("status form");
        let mut row = HashMap::new();
        row.insert("address".into(), "10.1.1.1%ether1".into());
        row.insert("area".into(), "backbone".into());
        row.insert("state".into(), "dr".into());
        row.insert("network-type".into(), "broadcast".into());
        row.insert("cost".into(), "10".into());
        row.insert("hello-interval".into(), "10s".into());
        row.insert("instance".into(), "default".into());
        let session = FormSession::edit(spec.id, "*1", &row, schema);
        let theme = DefaultTheme::new();
        let styles = Styles::from_palette(theme.palette());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| {
                render_form_sheet(frame, frame.area(), &session, schema, &styles);
            })
            .expect("draw");
        let buf = terminal.backend().buffer();
        let mut rendered = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                rendered.push_str(buf[(x, y)].symbol());
            }
        }
        assert!(rendered.contains("Address"), "{rendered}");
        assert!(rendered.contains("Network Type"), "{rendered}");
        assert!(rendered.contains("Hello Interval"), "{rendered}");
        assert!(rendered.contains("10.1.1.1%ether1"), "{rendered}");
        assert!(rendered.contains("instance"), "{rendered}");
        assert!(!rendered.contains("[1 Status]"), "{rendered}");
        assert!(rendered.contains("esc"), "{rendered}");
        assert!(
            rendered.contains("read only") || rendered.contains("ctrl+s"),
            "{rendered}"
        );
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
    fn prompt_with_defaults_is_not_dirty() {
        let mut values = HashMap::new();
        values.insert("file-system".into(), "ext4".into());
        let session = FormSession::prompt_with(
            "disks",
            "*d1",
            "format",
            &mtui_core::FORMAT_DISK_PROMPT,
            values,
        );
        assert!(!session.is_dirty());
        assert_eq!(
            session.values.get("file-system").map(String::as_str),
            Some("ext4")
        );
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
        assert_eq!(
            session.values.get("arp").map(String::as_str),
            Some("enabled")
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
        assert!(
            rendered.contains("type filter") || rendered.contains("enter select"),
            "picker hint must stay pinned: {rendered}"
        );
    }
}
