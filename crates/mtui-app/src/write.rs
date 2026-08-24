//! Resource write overlays and mutation commands.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::Path;

use mtui_core::{
    AT_CHAT_PROMPT, ActionCommand, ActionKind, ActionSpec, CERT_EXPORT_PROMPT, CERT_IMPORT_PROMPT,
    CERT_SIGN_PROMPT, DASHBOARD_ID, EXPORT_CONFIG_PROMPT, IMPORT_CONFIG_PROMPT,
    INSTALL_PACKAGE_PROMPT, INTERFACE_CREATE_TARGETS, RESET_CONFIG_PROMPT, SMS_PROMPT, WOL_PROMPT,
    action_label, patch_body, resource_by_id, supports_bulk_select, truthy,
};
use mtui_routeros::MASKED_VALUE;
use mtui_ui::{
    ActionMenuItem, ActionMenuState, COPY_FORM, FormSession, ProbeKind, ProbeState, Row, TorchState,
};

use crate::app::{App, AppCommand, Overlay, Pane};
use crate::event::WorkerMsg;
use crate::session::SessionId;

#[derive(Clone, PartialEq, Eq)]
pub enum MutationOp {
    Patch {
        endpoint: String,
        id: Option<String>,
        fields: BTreeMap<String, String>,
    },
    Put {
        endpoint: String,
        fields: BTreeMap<String, String>,
    },
    Delete {
        endpoint: String,
        id: String,
    },
    Command {
        endpoint: String,
        command: String,
        fields: BTreeMap<String, String>,
    },
    Batch {
        ops: Vec<MutationOp>,
    },
}

impl fmt::Debug for MutationOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Patch {
                endpoint,
                id,
                fields,
            } => f
                .debug_struct("Patch")
                .field("endpoint", endpoint)
                .field("id", id)
                .field("fields", &redact_mutation_fields(fields))
                .finish(),
            Self::Put { endpoint, fields } => f
                .debug_struct("Put")
                .field("endpoint", endpoint)
                .field("fields", &redact_mutation_fields(fields))
                .finish(),
            Self::Delete { endpoint, id } => f
                .debug_struct("Delete")
                .field("endpoint", endpoint)
                .field("id", id)
                .finish(),
            Self::Command {
                endpoint,
                command,
                fields,
            } => f
                .debug_struct("Command")
                .field("endpoint", endpoint)
                .field("command", command)
                .field("fields", &redact_mutation_fields(fields))
                .finish(),
            Self::Batch { ops } => f.debug_struct("Batch").field("ops", ops).finish(),
        }
    }
}

fn redact_mutation_fields(fields: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    fields
        .iter()
        .map(|(key, value)| {
            let sensitive = key.eq_ignore_ascii_case("password")
                || key.eq_ignore_ascii_case("contents")
                || key.contains("secret")
                || key.contains("passphrase");
            let shown = if sensitive {
                MASKED_VALUE.to_string()
            } else {
                value.clone()
            };
            (key.clone(), shown)
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmSession {
    pub title: String,
    pub body: String,
    pub action_id: String,
    pub command: ActionCommand,
    pub record_id: String,
    pub record_ids: Vec<String>,
    pub record_name: String,
    pub endpoint: String,
    pub fields: BTreeMap<String, String>,
}

impl App {
    pub(crate) fn current_actions(&self) -> Vec<&ActionSpec> {
        let Some(spec) = resource_by_id(&self.current_resource) else {
            return Vec::new();
        };
        spec.resolved_actions(self.table.selected_row())
    }

    pub(crate) fn footer_action_hints(&self) -> Vec<(String, String)> {
        if self.pane == Pane::Console {
            return vec![
                ("f".into(), "fullscreen".into()),
                ("/".into(), "search".into()),
                ("enter".into(), "expand".into()),
                ("h/l".into(), "json".into()),
                ("c".into(), "copy".into()),
                ("`".into(), "hide".into()),
                ("q".into(), "quit".into()),
            ];
        }
        let mut hints = vec![
            ("?".into(), "help".into()),
            ("i".into(), "about".into()),
            ("ctrl+k".into(), "commands".into()),
            ("`".into(), "console".into()),
            ("F4".into(), "safe mode".into()),
        ];
        if self.nav.show_hidden {
            if self.pane == Pane::Nav {
                hints.push(("-".into(), "restore".into()));
            }
            hints.push((".".into(), "done".into()));
        } else {
            if self.pane == Pane::Nav {
                hints.push(("-".into(), "hide".into()));
            }
            if !self.nav.hidden.is_empty() {
                hints.push((".".into(), "hidden".into()));
            }
        }
        if !self.session_ready() {
            hints.push(("r".into(), "reconnect".into()));
        } else if self.resource_actions_allowed() {
            let row = self.table.selected_row();
            for action in self
                .current_actions()
                .into_iter()
                .filter(|action| self.action_offered_in_pane(action))
                .take(5)
            {
                if let Some(key) = action.key {
                    hints.push((key.to_string(), action_label(action, row)));
                }
            }
            if self.pane_allows_row_actions()
                && resource_by_id(&self.current_resource)
                    .is_some_and(|spec| !spec.actions.is_empty())
            {
                hints.push(("a".into(), "actions".into()));
            }
        }
        if matches!(self.pane, Pane::Content | Pane::Inspector)
            && self.current_resource != "logs"
            && !self.status.starts_with("Filter:")
        {
            hints.push(("y".into(), "copy".into()));
            if self.pane == Pane::Content {
                hints.push(("Y".into(), "copy table".into()));
            }
        }
        if self.pane == Pane::Content
            && supports_bulk_select(&self.current_resource)
            && !self.status.starts_with("Filter:")
        {
            hints.push(("space".into(), "check".into()));
            hints.push(("*".into(), "all".into()));
        }
        hints.push(("r".into(), "refresh".into()));
        hints.push(("ctrl+t".into(), "tab".into()));
        hints.push(("q".into(), "quit".into()));
        hints
    }

    fn resource_actions_allowed(&self) -> bool {
        self.current_resource != "logs" && self.current_resource != DASHBOARD_ID
    }

    pub(crate) fn pane_allows_row_actions(&self) -> bool {
        matches!(self.pane, Pane::Content | Pane::Inspector)
    }

    pub(crate) fn action_offered_in_pane(&self, action: &ActionSpec) -> bool {
        if !self.resource_actions_allowed() || self.pane == Pane::Console {
            return false;
        }
        if action.needs_selection {
            self.pane_allows_row_actions()
        } else {
            matches!(self.pane, Pane::Nav | Pane::Content | Pane::Inspector)
        }
    }

    pub(crate) fn dispatch_named_action(&mut self, action_id: &str) -> Vec<AppCommand> {
        let Some(action) = self
            .current_actions()
            .into_iter()
            .find(|action| action.id == action_id)
            .copied()
        else {
            return Vec::new();
        };
        self.dispatch_action(&action)
    }

    pub(crate) fn dispatch_enter_action(&mut self) -> Vec<AppCommand> {
        if let Some(action) = self
            .current_actions()
            .into_iter()
            .find(|action| action.enter)
            .copied()
        {
            return self.dispatch_action(&action);
        }
        Vec::new()
    }

    pub(crate) fn action_key_consumed(&self, ch: char) -> bool {
        if ch == 'a' {
            return self.pane_allows_row_actions()
                && resource_by_id(&self.current_resource)
                    .is_some_and(|spec| !spec.actions.is_empty());
        }
        self.current_actions()
            .iter()
            .any(|action| action.key == Some(ch) && self.action_offered_in_pane(action))
    }

    pub(crate) fn dispatch_key_action(&mut self, key: char) -> Vec<AppCommand> {
        if key == 'a' {
            return self.open_action_menu();
        }
        let Some(action) = self
            .current_actions()
            .into_iter()
            .find(|action| action.key == Some(key))
            .copied()
        else {
            return Vec::new();
        };
        self.dispatch_action(&action)
    }

    fn dispatch_action(&mut self, action: &ActionSpec) -> Vec<AppCommand> {
        if self.deny_if_unavailable(action) {
            return Vec::new();
        }
        match action.kind {
            ActionKind::Edit => self.open_edit(),
            ActionKind::Create => self.open_create(&self.current_resource.clone()),
            ActionKind::Confirm { command } => self.open_confirm(action, command),
            ActionKind::Prompt { command } => self.open_prompt(command),
            ActionKind::Overlay { id: "torch" } => self.open_torch(),
            ActionKind::Overlay { id: "ping" } => self.open_probe(ProbeKind::Ping),
            ActionKind::Overlay { id: "traceroute" } => self.open_probe(ProbeKind::Traceroute),
            ActionKind::Overlay {
                id: "bandwidth-test",
            } => self.open_probe(ProbeKind::BandwidthTest),
            ActionKind::Overlay { id: "flood-ping" } => self.open_probe(ProbeKind::FloodPing),
            ActionKind::Overlay { id: "mac-scan" } => self.open_probe(ProbeKind::MacScan),
            ActionKind::Overlay { id: "ip-scan" } => self.open_probe(ProbeKind::IpScan),
            ActionKind::Overlay { id: "profiler" } => self.open_probe(ProbeKind::Profiler),
            ActionKind::Overlay { id: "wifi-scan" } => self.open_wifi_scan(),
            ActionKind::Overlay { id: "create-type" } => self.open_type_picker(),
            ActionKind::Overlay { .. } => Vec::new(),
        }
    }

    fn open_edit(&mut self) -> Vec<AppCommand> {
        let Some(spec) = resource_by_id(&self.current_resource) else {
            return Vec::new();
        };
        let Some(schema) = spec.form else {
            self.status = "This screen has no editor".into();
            return Vec::new();
        };
        let Some(row) = self.table.selected_row().cloned() else {
            return Vec::new();
        };
        let id = row.get(".id").cloned().unwrap_or_default();
        if !spec.is_singleton() && id.is_empty() {
            self.status = "Selected row has no id".into();
            return Vec::new();
        }
        self.overlay = Overlay::Form(FormSession::edit(spec.id, id, &row, schema));
        tracing::trace!(resource_id = spec.id, overlay = "form", "opened pane");
        Vec::new()
    }

    pub(crate) fn open_create(&mut self, resource_id: &str) -> Vec<AppCommand> {
        let Some(spec) = resource_by_id(resource_id) else {
            return Vec::new();
        };
        let Some(schema) = spec.form else {
            self.status = "This screen cannot create records".into();
            return Vec::new();
        };
        self.overlay = Overlay::Form(FormSession::create(spec.id, schema));
        Vec::new()
    }

    fn open_prompt(&mut self, command: ActionCommand) -> Vec<AppCommand> {
        match command {
            ActionCommand::BackupSave => self.open_backup_save_prompt(),
            ActionCommand::Upload => self.open_file_upload_prompt(),
            ActionCommand::Download => self.open_file_download_prompt(),
            ActionCommand::Fetch => self.open_file_fetch_prompt(),
            ActionCommand::Copy
            | ActionCommand::Sign
            | ActionCommand::Import
            | ActionCommand::ExportCertificate
            | ActionCommand::Export
            | ActionCommand::Install
            | ActionCommand::ResetConfiguration
            | ActionCommand::WakeOnLan
            | ActionCommand::SendSms
            | ActionCommand::AtChat => self.open_schema_prompt(command),
            _ => Vec::new(),
        }
    }

    fn open_file_upload_prompt(&mut self) -> Vec<AppCommand> {
        let mut values = HashMap::new();
        values.insert("local-path".into(), String::new());
        values.insert("remote-name".into(), String::new());
        self.overlay = Overlay::Form(FormSession::prompt_fields(
            self.current_resource.clone(),
            String::new(),
            "upload",
            &mtui_ui::UPLOAD_FORM,
            values,
        ));
        Vec::new()
    }

    fn open_file_download_prompt(&mut self) -> Vec<AppCommand> {
        let Some(row) = self.table.selected_row() else {
            return Vec::new();
        };
        let id = row.get(".id").cloned().unwrap_or_default();
        let mut values = HashMap::new();
        values.insert("local-path".into(), String::new());
        if let Some(contents) = row.get("contents").filter(|value| !value.is_empty()) {
            values.insert("contents".into(), contents.clone());
        }
        self.overlay = Overlay::Form(FormSession::prompt_fields(
            self.current_resource.clone(),
            id,
            "download",
            &mtui_ui::DOWNLOAD_FORM,
            values,
        ));
        Vec::new()
    }

    fn open_file_fetch_prompt(&mut self) -> Vec<AppCommand> {
        let mut values = HashMap::new();
        values.insert("url".into(), String::new());
        values.insert("dst-path".into(), String::new());
        values.insert("user".into(), String::new());
        values.insert("password".into(), String::new());
        self.overlay = Overlay::Form(FormSession::prompt_fields(
            self.current_resource.clone(),
            String::new(),
            "fetch",
            &mtui_ui::FETCH_FORM,
            values,
        ));
        Vec::new()
    }

    fn open_schema_prompt(&mut self, command: ActionCommand) -> Vec<AppCommand> {
        let (schema, needs_row) = match command {
            ActionCommand::Copy => (&COPY_FORM, true),
            ActionCommand::Sign => (&CERT_SIGN_PROMPT, true),
            ActionCommand::Import => {
                if self.current_resource == "files" {
                    (&IMPORT_CONFIG_PROMPT, false)
                } else {
                    (&CERT_IMPORT_PROMPT, false)
                }
            }
            ActionCommand::ExportCertificate => (&CERT_EXPORT_PROMPT, true),
            ActionCommand::Export => (&EXPORT_CONFIG_PROMPT, false),
            ActionCommand::Install => (&INSTALL_PACKAGE_PROMPT, false),
            ActionCommand::ResetConfiguration => (&RESET_CONFIG_PROMPT, false),
            ActionCommand::WakeOnLan => (&WOL_PROMPT, false),
            ActionCommand::SendSms => (&SMS_PROMPT, false),
            ActionCommand::AtChat => (&AT_CHAT_PROMPT, true),
            _ => return Vec::new(),
        };
        let (id, name) = if needs_row {
            let Some(row) = self.table.selected_row() else {
                return Vec::new();
            };
            let id = row.get(".id").cloned().unwrap_or_default();
            let name = row
                .get("name")
                .or_else(|| row.get("interface"))
                .cloned()
                .unwrap_or_else(|| id.clone());
            (id, name)
        } else {
            (String::new(), String::new())
        };
        let mut values = HashMap::new();
        if command == ActionCommand::Copy {
            values.insert("new-name".into(), format!("{name}-copy"));
        }
        if command == ActionCommand::ExportCertificate {
            values.insert("type".into(), "pem".into());
        }
        self.overlay = Overlay::Form(FormSession::prompt_with(
            self.current_resource.clone(),
            id,
            command.rest_name(),
            schema,
            values,
        ));
        Vec::new()
    }

    fn open_backup_save_prompt(&mut self) -> Vec<AppCommand> {
        let mut values = HashMap::new();
        values.insert("name".into(), String::new());
        values.insert("password".into(), String::new());
        self.overlay = Overlay::Form(FormSession::prompt_fields(
            self.current_resource.clone(),
            String::new(),
            ActionCommand::BackupSave.rest_name(),
            &mtui_ui::BACKUP_SAVE_FORM,
            values,
        ));
        Vec::new()
    }

    fn open_confirm(&mut self, action: &ActionSpec, command: ActionCommand) -> Vec<AppCommand> {
        match command {
            ActionCommand::MoveUp => return self.move_selected(-1),
            ActionCommand::MoveDown => return self.move_selected(1),
            _ => {}
        }
        let Some(spec) = resource_by_id(&self.current_resource) else {
            return Vec::new();
        };
        let row = self.table.selected_row();
        if action.needs_selection && row.is_none() {
            return Vec::new();
        }
        if matches!(command, ActionCommand::ToggleDisabled) && row.is_none() {
            return Vec::new();
        }
        let checked = if supports_bulk_select(&self.current_resource) && bulk_command(command) {
            self.table.checked_ids()
        } else {
            Vec::new()
        };
        let mut record_id = row
            .and_then(|row| row.get(".id"))
            .cloned()
            .unwrap_or_default();
        let mut record_name = row
            .and_then(|row| {
                row.get("name")
                    .or_else(|| row.get("interface"))
                    .or_else(|| row.get("address"))
            })
            .cloned()
            .unwrap_or_else(|| record_id.clone());
        let record_ids = match checked.as_slice() {
            [] => Vec::new(),
            [id] => {
                record_id.clone_from(id);
                checked
            }
            [id, ..] => {
                record_id.clone_from(id);
                record_name = format!("{} items", checked.len());
                checked
            }
        };
        if !action.needs_selection {
            record_id.clear();
        }
        let command = match (command, row) {
            (ActionCommand::ToggleDisabled, _) if record_ids.len() > 1 => {
                ActionCommand::ToggleDisabled
            }
            (ActionCommand::ToggleDisabled, Some(row)) => {
                if truthy(row.get("disabled").map(String::as_str)) {
                    ActionCommand::Enable
                } else {
                    ActionCommand::Disable
                }
            }
            (ActionCommand::ToggleDisabled, None) => return Vec::new(),
            (other, _) => other,
        };
        let mut fields = BTreeMap::new();
        if action.id == "backup-load" {
            if record_name.is_empty() {
                self.status = "Backup load needs a file name".into();
                return Vec::new();
            }
            fields.insert("name".into(), record_name.clone());
        }
        let label = action_label(action, row);
        let mut body = confirm_body(action.id, &label, &record_name);
        if self.safe_mode.we_hold()
            && matches!(
                action.id,
                "reboot" | "shutdown" | "upgrade" | "reset-configuration" | "backup-load"
            )
        {
            body.push_str(
                "\n\nSafe Mode cannot undo reboot, shutdown, firmware upgrade, or reset.",
            );
        }
        self.overlay = Overlay::Confirm(ConfirmSession {
            title: label,
            body,
            action_id: action.id.to_string(),
            command,
            record_id,
            record_ids,
            record_name,
            endpoint: command_base_path(action.id, spec.endpoint()),
            fields,
        });
        tracing::trace!(overlay = "confirm", action = action.id, "opened pane");
        Vec::new()
    }

    fn move_selected(&mut self, delta: isize) -> Vec<AppCommand> {
        let Some(spec) = resource_by_id(&self.current_resource) else {
            return Vec::new();
        };
        let idx = self.table.selected;
        let Some(dest_idx) = idx.checked_add_signed(delta) else {
            self.status = "Already first".into();
            return Vec::new();
        };
        let visible = self.table.visible_rows();
        if dest_idx >= visible.len() {
            self.status = "Already last".into();
            return Vec::new();
        }
        let selected_id = visible
            .get(idx)
            .and_then(|row| row.get(".id"))
            .cloned()
            .unwrap_or_default();
        let destination = visible
            .get(dest_idx)
            .and_then(|row| row.get(".id"))
            .cloned()
            .unwrap_or_default();
        if selected_id.is_empty() || destination.is_empty() {
            self.status = "Selected row has no id".into();
            return Vec::new();
        }
        let mut fields = BTreeMap::new();
        fields.insert(".id".into(), selected_id);
        fields.insert("destination".into(), destination);
        self.status = "Moving…".into();
        vec![self.mutate_command(MutationOp::Command {
            endpoint: spec.endpoint().to_string(),
            command: ActionCommand::MoveUp.rest_name().to_string(),
            fields,
        })]
    }

    fn open_action_menu(&mut self) -> Vec<AppCommand> {
        let row = self.table.selected_row();
        let items: Vec<ActionMenuItem> = self
            .current_actions()
            .into_iter()
            .map(|action| {
                let blocked = self
                    .access
                    .action_block_reason(&self.current_resource, action);
                ActionMenuItem {
                    id: action.id.to_string(),
                    label: action_label(action, row),
                    keys: action.key.map_or_else(String::new, |key| key.to_string()),
                    danger: action.danger,
                    note: blocked.map_or(String::new(), |_| "blocked".into()),
                }
            })
            .collect();
        if items.is_empty() {
            return Vec::new();
        }
        self.overlay = Overlay::ActionMenu(ActionMenuState::new(items));
        tracing::trace!(overlay = "action-menu", "opened pane");
        Vec::new()
    }

    fn open_type_picker(&mut self) -> Vec<AppCommand> {
        let items = INTERFACE_CREATE_TARGETS
            .iter()
            .map(|(id, label)| ActionMenuItem {
                id: (*id).to_string(),
                label: (*label).to_string(),
                keys: String::new(),
                danger: false,
                note: String::new(),
            })
            .collect();
        self.overlay = Overlay::TypePicker(ActionMenuState::new(items));
        tracing::trace!(overlay = "type-picker", "opened pane");
        Vec::new()
    }

    fn open_torch(&mut self) -> Vec<AppCommand> {
        let Some(row) = self.table.selected_row() else {
            return Vec::new();
        };
        let name = row.get("name").cloned().unwrap_or_default();
        let id = row.get(".id").cloned().unwrap_or_default();
        if name.is_empty() {
            self.status = "Torch needs an interface name".into();
            return Vec::new();
        }
        self.torch_generation = self.torch_generation.wrapping_add(1);
        self.overlay = Overlay::Torch(TorchState::new(name, id, self.torch_generation));
        tracing::trace!(overlay = "torch", "opened pane");
        Vec::new()
    }

    fn open_probe(&mut self, kind: ProbeKind) -> Vec<AppCommand> {
        self.probe_generation = self.probe_generation.wrapping_add(1);
        self.overlay = Overlay::Probe(ProbeState::new(kind, self.probe_generation));
        tracing::trace!(overlay = kind.command(), "opened pane");
        Vec::new()
    }

    fn open_wifi_scan(&mut self) -> Vec<AppCommand> {
        let Some(row) = self.table.selected_row() else {
            return Vec::new();
        };
        let name = row.get("name").cloned().unwrap_or_default();
        if name.is_empty() {
            self.status = "Scan needs an interface name".into();
            return Vec::new();
        }
        self.probe_generation = self.probe_generation.wrapping_add(1);
        let mut probe = ProbeState::new(ProbeKind::WifiScan, self.probe_generation);
        probe.src = name;
        self.overlay = Overlay::Probe(probe);
        Vec::new()
    }

    fn save_prompt_form(&mut self, command: &'static str) -> Vec<AppCommand> {
        let Overlay::Form(session) = &self.overlay else {
            return Vec::new();
        };
        let resource_id = session.resource_id.clone();
        let record_id = session.record_id.clone();
        let values = session.values.clone();
        let Some(spec) = resource_by_id(&resource_id) else {
            return Vec::new();
        };
        let (endpoint, fields, status) = if command == ActionCommand::BackupSave.rest_name() {
            let mut fields = BTreeMap::new();
            let Some(name) = values
                .get("name")
                .map(String::as_str)
                .map(str::trim)
                .filter(|name| !name.is_empty())
            else {
                if let Overlay::Form(session) = &mut self.overlay {
                    session.error = Some("Name is required".into());
                }
                return Vec::new();
            };
            fields.insert("name".into(), name.to_string());
            if let Some(password) = values
                .get("password")
                .map(String::as_str)
                .filter(|password| !password.is_empty())
            {
                fields.insert("password".into(), password.to_string());
            }
            (
                command_base_path("backup-save", spec.endpoint()),
                fields,
                "Saving backup…",
            )
        } else if command == ActionCommand::Copy.rest_name() {
            let mut fields = BTreeMap::new();
            fields.insert(".id".into(), record_id);
            if let Some(name) = values.get("new-name") {
                fields.insert("new-name".into(), name.clone());
            }
            (spec.endpoint().to_string(), fields, "Copying…")
        } else {
            let schema = match &self.overlay {
                Overlay::Form(session) => session.prompt_schema,
                _ => None,
            };
            let mut fields = BTreeMap::new();
            if !record_id.is_empty() {
                fields.insert(".id".into(), record_id);
            }
            if let Some(schema) = schema {
                for key in schema.writable_keys() {
                    let Some(value) = values.get(key) else {
                        continue;
                    };
                    if value == MASKED_VALUE || value.is_empty() {
                        continue;
                    }
                    fields.insert(key.to_string(), value.clone());
                }
            }
            let action_id = match command {
                "export" if resource_id == "files" => "export-config",
                "import" if resource_id == "files" => "import-config",
                "wol" => "wol",
                "send" if resource_id == "sms" => "sms",
                "reset-configuration" => "reset-configuration",
                other => other,
            };
            (
                command_base_path(action_id, spec.endpoint()),
                fields,
                "Running…",
            )
        };
        if let Overlay::Form(session) = &mut self.overlay {
            session.saving = true;
        }
        self.status = status.into();
        vec![self.mutate_command(MutationOp::Command {
            endpoint,
            command: command.to_string(),
            fields,
        })]
    }

    pub(crate) fn save_form(&mut self) -> Vec<AppCommand> {
        if !self.session_ready() {
            self.status = self.link_status_message();
            return Vec::new();
        }
        let (saving, is_prompt) = match &self.overlay {
            Overlay::Form(session) => (session.saving, session.prompt_command.is_some()),
            _ => return Vec::new(),
        };
        if saving {
            return Vec::new();
        }
        if is_prompt {
            let Overlay::Form(session) = &self.overlay else {
                return Vec::new();
            };
            let Some(command) = session.prompt_command else {
                return Vec::new();
            };
            return self.save_prompt(command);
        }
        let previewing = matches!(&self.overlay, Overlay::Form(session) if session.confirm_save);
        if !previewing {
            return self.show_save_preview();
        }
        self.commit_form_save()
    }

    fn show_save_preview(&mut self) -> Vec<AppCommand> {
        let Overlay::Form(session) = &self.overlay else {
            return Vec::new();
        };
        let Some(spec) = resource_by_id(&session.resource_id) else {
            return Vec::new();
        };
        let Some(schema) = spec.form else {
            return Vec::new();
        };
        let mut body = patch_body(schema, &session.original, &session.values, MASKED_VALUE);
        if session.mode == mtui_ui::FormMode::Create {
            body.retain(|_, value| !value.is_empty());
            if body.is_empty() {
                if let Overlay::Form(session) = &mut self.overlay {
                    session.error = Some("Fill required fields".into());
                }
                return Vec::new();
            }
        } else if body.is_empty() {
            self.overlay = Overlay::None;
            self.status = "No changes".into();
            return Vec::new();
        }
        let count = body.len();
        if let Overlay::Form(session) = &mut self.overlay {
            session.confirm_save = true;
            session.error = None;
        }
        self.status = format!("Save preview · {count} field(s)");
        Vec::new()
    }

    fn commit_form_save(&mut self) -> Vec<AppCommand> {
        let Overlay::Form(session) = &self.overlay else {
            return Vec::new();
        };
        let Some(spec) = resource_by_id(&session.resource_id) else {
            return Vec::new();
        };
        let Some(schema) = spec.form else {
            return Vec::new();
        };
        let mut body = patch_body(schema, &session.original, &session.values, MASKED_VALUE);
        if session.mode == mtui_ui::FormMode::Create {
            body.retain(|_, value| !value.is_empty());
            if body.is_empty() {
                if let Overlay::Form(session) = &mut self.overlay {
                    session.error = Some("Fill required fields".into());
                }
                return Vec::new();
            }
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = true;
                session.error = None;
            }
            self.status = "Creating…".into();
            return vec![self.mutate_command(MutationOp::Put {
                endpoint: spec.endpoint().to_string(),
                fields: body,
            })];
        }
        if body.is_empty() {
            self.overlay = Overlay::None;
            self.status = "No changes".into();
            return Vec::new();
        }
        let id = if spec.is_singleton() {
            None
        } else {
            Some(session.record_id.clone())
        };
        if let Overlay::Form(session) = &mut self.overlay {
            session.saving = true;
            session.error = None;
        }
        self.status = "Saving…".into();
        vec![self.mutate_command(MutationOp::Patch {
            endpoint: spec.endpoint().to_string(),
            id,
            fields: body,
        })]
    }

    fn save_prompt(&mut self, command: &'static str) -> Vec<AppCommand> {
        match command {
            "upload" => self.save_upload_prompt(),
            "download" => self.save_download_prompt(),
            "fetch" => self.save_fetch_prompt(),
            "sign" | "export-certificate" => self.save_cert_prompt(),
            "import" if self.current_resource != "files" => self.save_cert_prompt(),
            _ => self.save_prompt_form(command),
        }
    }

    fn save_upload_prompt(&mut self) -> Vec<AppCommand> {
        let Overlay::Form(session) = &self.overlay else {
            return Vec::new();
        };
        let path = session
            .values
            .get("local-path")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if path.is_empty() {
            if let Overlay::Form(session) = &mut self.overlay {
                session.error = Some("Local path is required".into());
            }
            return Vec::new();
        }
        let mut remote_name = session
            .values
            .get("remote-name")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if remote_name.is_empty() {
            remote_name = Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("file")
                .to_string();
        }
        if let Overlay::Form(session) = &mut self.overlay {
            session.saving = true;
            session.error = None;
        }
        self.status = "Reading local file…".into();
        vec![AppCommand::ReadLocalFile {
            session: SessionId::UNSTAMPED,
            request_id: self.next_request(),
            generation: self.poll_generation,
            path,
            remote_name,
        }]
    }

    fn save_download_prompt(&mut self) -> Vec<AppCommand> {
        let Overlay::Form(session) = &self.overlay else {
            return Vec::new();
        };
        let path = session
            .values
            .get("local-path")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if path.is_empty() {
            if let Overlay::Form(session) = &mut self.overlay {
                session.error = Some("Local path is required".into());
            }
            return Vec::new();
        }
        let contents = session
            .values
            .get("contents")
            .filter(|value| !value.is_empty())
            .cloned();
        let record_id = session.record_id.clone();
        let endpoint = resource_by_id(&session.resource_id)
            .map_or_else(|| "/rest/file".into(), |spec| spec.endpoint().to_string());
        if let Overlay::Form(session) = &mut self.overlay {
            session.saving = true;
            session.error = None;
        }
        if let Some(contents) = contents {
            self.status = "Writing local file…".into();
            return vec![AppCommand::WriteLocalFile {
                session: SessionId::UNSTAMPED,
                request_id: self.next_request(),
                generation: self.poll_generation,
                path,
                contents,
            }];
        }
        if record_id.is_empty() {
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = false;
                session.error = Some(
                    "Classic API did not return file contents; use Fetch URL or copy another way."
                        .into(),
                );
            }
            self.status =
                "Classic API did not return file contents; use Fetch URL or copy another way."
                    .into();
            return Vec::new();
        }
        self.status = "Fetching file…".into();
        vec![AppCommand::FetchRecord {
            session: SessionId::UNSTAMPED,
            request_id: self.next_request(),
            generation: self.poll_generation,
            endpoint,
            id: record_id,
            local_path: path,
        }]
    }

    fn save_fetch_prompt(&mut self) -> Vec<AppCommand> {
        let Overlay::Form(session) = &self.overlay else {
            return Vec::new();
        };
        let url = session
            .values
            .get("url")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if url.is_empty() {
            if let Overlay::Form(session) = &mut self.overlay {
                session.error = Some("URL is required".into());
            }
            return Vec::new();
        }
        let dst_path = session
            .values
            .get("dst-path")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let user = session
            .values
            .get("user")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let password = session.values.get("password").cloned().unwrap_or_default();
        let mut fields = BTreeMap::new();
        fields.insert("url".into(), url.clone());
        if !dst_path.is_empty() {
            fields.insert("dst-path".into(), dst_path);
        }
        if !user.is_empty() {
            fields.insert("user".into(), user);
        }
        if !password.is_empty() && password != MASKED_VALUE {
            fields.insert("password".into(), password);
        }
        tracing::info!(url = url.as_str(), "tool fetch");
        if let Overlay::Form(session) = &mut self.overlay {
            session.saving = true;
            session.error = None;
        }
        self.status = "Fetching…".into();
        vec![self.mutate_command(MutationOp::Command {
            endpoint: "/rest/tool".into(),
            command: "fetch".into(),
            fields,
        })]
    }

    fn save_cert_prompt(&mut self) -> Vec<AppCommand> {
        let (command, schema, record_id, values, resource_id) = match &self.overlay {
            Overlay::Form(session) => {
                let Some(command) = session.prompt_command else {
                    return Vec::new();
                };
                (
                    command,
                    session.prompt_schema.unwrap_or(&COPY_FORM),
                    session.record_id.clone(),
                    session.values.clone(),
                    session.resource_id.clone(),
                )
            }
            _ => return Vec::new(),
        };
        let mut fields = BTreeMap::new();
        if command != "import" {
            if record_id.is_empty() {
                self.form_error("Selected row has no id");
                return Vec::new();
            }
            fields.insert(".id".into(), record_id);
        }
        for key in schema.writable_keys() {
            let Some(value) = values.get(key) else {
                continue;
            };
            if value == MASKED_VALUE || value.is_empty() {
                continue;
            }
            fields.insert(key.to_string(), value.clone());
        }
        let missing = match command {
            "sign" if !fields.contains_key("ca") => Some("CA name is required"),
            "import" | "export-certificate" if !fields.contains_key("file-name") => {
                Some("File name is required")
            }
            _ => None,
        };
        if let Some(error) = missing {
            self.form_error(error);
            return Vec::new();
        }
        let Some(spec) = resource_by_id(&resource_id) else {
            return Vec::new();
        };
        if let Overlay::Form(session) = &mut self.overlay {
            session.saving = true;
            session.error = None;
        }
        self.status = match command {
            "sign" => "Signing…".into(),
            "import" => "Importing…".into(),
            "export-certificate" => "Exporting…".into(),
            _ => "Copying…".into(),
        };
        vec![self.mutate_command(MutationOp::Command {
            endpoint: spec.endpoint().to_string(),
            command: command.to_string(),
            fields,
        })]
    }

    fn form_error(&mut self, error: &str) {
        if let Overlay::Form(session) = &mut self.overlay {
            session.error = Some(error.into());
        }
    }

    pub(crate) fn confirm_pending(&mut self) -> Vec<AppCommand> {
        if !self.session_ready() {
            self.status = self.link_status_message();
            return Vec::new();
        }
        let Overlay::Confirm(session) = &self.overlay else {
            return Vec::new();
        };
        let ids = if session.record_ids.len() > 1 {
            session.record_ids.clone()
        } else if !session.record_id.is_empty() {
            vec![session.record_id.clone()]
        } else {
            Vec::new()
        };
        if ids.len() > 1 {
            let ops = ids.iter().map(|id| self.bulk_op_for(session, id)).collect();
            self.status = format!("{} {}…", session.title, ids.len());
            self.overlay = Overlay::None;
            self.table.clear_checked();
            return vec![self.mutate_command(MutationOp::Batch { ops })];
        }
        let mut fields = session.fields.clone();
        if !session.record_id.is_empty() && !fields.contains_key(".id") {
            fields.insert(".id".into(), session.record_id.clone());
        }
        let endpoint = session.endpoint.clone();
        let op = match session.command {
            ActionCommand::Remove => MutationOp::Delete {
                endpoint,
                id: session.record_id.clone(),
            },
            other => MutationOp::Command {
                endpoint,
                command: other.rest_name().to_string(),
                fields,
            },
        };
        self.status = format!("{}…", session.title);
        self.overlay = Overlay::None;
        vec![self.mutate_command(op)]
    }

    fn bulk_op_for(&self, session: &ConfirmSession, id: &str) -> MutationOp {
        let endpoint = session.endpoint.clone();
        if session.command == ActionCommand::Remove {
            return MutationOp::Delete {
                endpoint,
                id: id.to_string(),
            };
        }
        let command = match session.command {
            ActionCommand::ToggleDisabled => {
                let disabled = self.table.rows.iter().any(|row| {
                    row.get(".id").map(String::as_str) == Some(id)
                        && truthy(row.get("disabled").map(String::as_str))
                });
                if disabled {
                    ActionCommand::Enable
                } else {
                    ActionCommand::Disable
                }
            }
            other => other,
        };
        let mut fields = session.fields.clone();
        fields.insert(".id".into(), id.to_string());
        MutationOp::Command {
            endpoint,
            command: command.rest_name().to_string(),
            fields,
        }
    }

    pub(crate) fn mutate_command(&mut self, op: MutationOp) -> AppCommand {
        AppCommand::Mutate {
            session: SessionId::UNSTAMPED,
            request_id: self.next_request(),
            generation: self.poll_generation,
            op,
        }
    }

    pub(crate) fn apply_mutate_result(&mut self, msg: WorkerMsg) -> Vec<AppCommand> {
        let WorkerMsg::MutateResult {
            generation, error, ..
        } = msg
        else {
            return Vec::new();
        };
        if generation != self.poll_generation {
            return Vec::new();
        }
        if let Some(cmds) = self.finish_safe_mode_mutate(error.as_deref()) {
            return cmds;
        }
        if let Some(err) = error {
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = false;
                session.error = Some(err.clone());
            }
            self.status = format!("Write failed: {}", Self::classify_write_error(&err));
            return Vec::new();
        }
        self.overlay = Overlay::None;
        self.status = "Saved".into();
        self.refreshing = true;
        self.poll_current()
    }

    pub(crate) fn apply_read_local_file(&mut self, msg: WorkerMsg) -> Vec<AppCommand> {
        let WorkerMsg::ReadLocalFileResult {
            generation,
            remote_name,
            contents,
            error,
            ..
        } = msg
        else {
            return Vec::new();
        };
        if generation != self.poll_generation {
            return Vec::new();
        }
        if let Some(err) = error {
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = false;
                session.error = Some(err.clone());
            }
            self.status = format!("Upload failed: {err}");
            return Vec::new();
        }
        let Some(contents) = contents else {
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = false;
                session.error = Some("file was empty".into());
            }
            return Vec::new();
        };
        let mut fields = BTreeMap::new();
        fields.insert("name".into(), remote_name);
        fields.insert("contents".into(), contents);
        tracing::info!("file upload");
        self.status = "Uploading…".into();
        vec![self.mutate_command(MutationOp::Put {
            endpoint: "/rest/file".into(),
            fields,
        })]
    }

    pub(crate) fn apply_write_local_file(&mut self, msg: WorkerMsg) -> Vec<AppCommand> {
        let WorkerMsg::WriteLocalFileResult {
            generation, error, ..
        } = msg
        else {
            return Vec::new();
        };
        if generation != self.poll_generation {
            return Vec::new();
        }
        if let Some(err) = error {
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = false;
                session.error = Some(err.clone());
            }
            self.status = format!("Download failed: {err}");
            return Vec::new();
        }
        self.overlay = Overlay::None;
        self.status = "Downloaded".into();
        Vec::new()
    }

    pub(crate) fn apply_record_result(&mut self, msg: WorkerMsg) -> Vec<AppCommand> {
        let WorkerMsg::RecordResult {
            generation,
            local_path,
            contents,
            error,
            ..
        } = msg
        else {
            return Vec::new();
        };
        if generation != self.poll_generation {
            return Vec::new();
        }
        if let Some(err) = error {
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = false;
                session.error = Some(err.clone());
            }
            self.status = format!("Download failed: {err}");
            return Vec::new();
        }
        let Some(contents) = contents.filter(|value| !value.is_empty()) else {
            let message =
                "Classic API did not return file contents; use Fetch URL or copy another way.";
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = false;
                session.error = Some(message.into());
            }
            self.status = message.into();
            return Vec::new();
        };
        self.status = "Writing local file…".into();
        vec![AppCommand::WriteLocalFile {
            session: SessionId::UNSTAMPED,
            request_id: self.next_request(),
            generation: self.poll_generation,
            path: local_path,
            contents,
        }]
    }

    pub(crate) fn apply_torch_result(
        &mut self,
        generation: u64,
        rows: Vec<HashMap<String, String>>,
        error: Option<String>,
        done: bool,
    ) -> Vec<AppCommand> {
        let Overlay::Torch(torch) = &mut self.overlay else {
            return Vec::new();
        };
        if generation != torch.generation {
            return Vec::new();
        }
        if let Some(err) = error {
            torch.error = Some(err);
            torch.running = false;
            return Vec::new();
        }
        torch.error = None;
        torch.push_samples(rows);
        if done {
            torch.running = false;
        }
        Vec::new()
    }

    pub(crate) fn apply_probe_result(
        &mut self,
        generation: u64,
        rows: Vec<HashMap<String, String>>,
        error: Option<String>,
        done: bool,
    ) -> Vec<AppCommand> {
        let Overlay::Probe(probe) = &mut self.overlay else {
            return Vec::new();
        };
        if generation != probe.generation {
            return Vec::new();
        }
        if let Some(err) = error {
            probe.running = false;
            probe.error = Some(err);
            return Vec::new();
        }
        probe.error = None;
        probe.push_samples(rows);
        if done {
            probe.running = false;
        }
        Vec::new()
    }

    pub(crate) fn start_probe(&mut self) -> Vec<AppCommand> {
        let rejected = {
            let Overlay::Probe(probe) = &mut self.overlay else {
                return Vec::new();
            };
            let kind = probe.kind;
            if kind.requires_address() && probe.address.trim().is_empty() {
                probe.error = Some("Address is required".into());
                true
            } else if kind.requires_interface() && probe.src.trim().is_empty() {
                probe.error = Some("Interface is required".into());
                true
            } else {
                false
            }
        };
        if rejected {
            self.status = match &self.overlay {
                Overlay::Probe(probe) => probe
                    .error
                    .clone()
                    .unwrap_or_else(|| "Address is required".into()),
                _ => "Address is required".into(),
            };
            return Vec::new();
        }
        self.probe_generation = self.probe_generation.wrapping_add(1);
        let generation = self.probe_generation;
        let Overlay::Probe(probe) = &mut self.overlay else {
            return Vec::new();
        };
        probe.generation = generation;
        probe.running = true;
        probe.error = None;
        let kind = probe.kind;
        let count = {
            let trimmed = probe.count.trim();
            if trimmed.is_empty() {
                kind.default_count().to_string()
            } else {
                trimmed.to_string()
            }
        };
        let protocol = {
            let trimmed = probe.protocol.trim();
            if trimmed.is_empty() {
                match kind {
                    ProbeKind::Traceroute => "icmp".into(),
                    ProbeKind::BandwidthTest => "tcp".into(),
                    _ => String::new(),
                }
            } else {
                trimmed.to_string()
            }
        };
        let address = probe.address.trim().to_string();
        let src = probe.src.trim().to_string();
        let request_id = self.next_request();
        match kind {
            ProbeKind::Ping => vec![AppCommand::FetchPing {
                session: SessionId::UNSTAMPED,
                request_id,
                generation,
                address,
                count,
                src,
            }],
            ProbeKind::Traceroute => vec![AppCommand::FetchTraceroute {
                session: SessionId::UNSTAMPED,
                request_id,
                generation,
                address,
                count,
                src,
                protocol,
            }],
            other => vec![AppCommand::FetchProbe {
                session: SessionId::UNSTAMPED,
                request_id,
                generation,
                endpoint: other.endpoint().to_string(),
                command: other.command().to_string(),
                fields: probe_fields(other, &address, &count, &src, &protocol),
            }],
        }
    }

    pub(crate) fn torch_sample_command(&mut self) -> Vec<AppCommand> {
        let Some((generation, interface, src, dst, protocol, port)) = ({
            match &self.overlay {
                Overlay::Torch(torch) if torch.running => Some((
                    torch.generation,
                    torch.interface.clone(),
                    torch.src.clone(),
                    torch.dst.clone(),
                    torch.protocol.clone(),
                    torch.port.clone(),
                )),
                _ => None,
            }
        }) else {
            return Vec::new();
        };
        vec![AppCommand::FetchTorch {
            session: SessionId::UNSTAMPED,
            request_id: self.next_request(),
            generation,
            interface,
            src,
            dst,
            protocol,
            port,
        }]
    }

    pub(crate) fn row_to_display(rows: Vec<mtui_routeros::Resource>) -> Vec<Row> {
        rows.into_iter().map(|row| row.display_row()).collect()
    }
}

fn probe_fields(
    kind: ProbeKind,
    address: &str,
    count: &str,
    src: &str,
    protocol: &str,
) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    match kind {
        ProbeKind::Ping | ProbeKind::Traceroute | ProbeKind::FloodPing => {
            fields.insert("address".into(), address.to_string());
            if !count.is_empty() {
                fields.insert("count".into(), count.to_string());
            }
            if !src.is_empty() {
                fields.insert("src-address".into(), src.to_string());
            }
            if kind == ProbeKind::Traceroute && !protocol.is_empty() {
                fields.insert("protocol".into(), protocol.to_string());
            }
        }
        ProbeKind::BandwidthTest => {
            fields.insert("address".into(), address.to_string());
            if !count.is_empty() {
                fields.insert("duration".into(), count.to_string());
            }
            if !protocol.is_empty() {
                fields.insert("protocol".into(), protocol.to_string());
            }
        }
        ProbeKind::MacScan | ProbeKind::WifiScan => {
            if !src.is_empty() {
                let key = if kind == ProbeKind::WifiScan {
                    "numbers"
                } else {
                    "interface"
                };
                fields.insert(key.into(), src.to_string());
            }
        }
        ProbeKind::IpScan => {
            fields.insert("address".into(), address.to_string());
            if !src.is_empty() {
                fields.insert("interface".into(), src.to_string());
            }
        }
        ProbeKind::Profiler => {
            if !count.is_empty() {
                fields.insert("duration".into(), count.to_string());
            }
        }
    }
    fields
}

fn command_base_path(action_id: &str, resource_endpoint: &str) -> String {
    match action_id {
        "reboot" | "shutdown" | "reset-configuration" => "/rest/system".into(),
        "backup-save" | "backup-load" => "/rest/system/backup".into(),
        "upgrade" => "/rest/system/routerboard".into(),
        "export-config" | "export" | "import-config" => "/rest".into(),
        "check-for-updates" => "/rest/system/package/update".into(),
        "wol" | "wake-on-lan" => "/rest/tool".into(),
        "sms" | "send-sms" => "/rest/tool/sms".into(),
        _ => resource_endpoint.to_string(),
    }
}

fn bulk_command(command: ActionCommand) -> bool {
    matches!(
        command,
        ActionCommand::Enable
            | ActionCommand::Disable
            | ActionCommand::ToggleDisabled
            | ActionCommand::Remove
            | ActionCommand::ResetCounters
            | ActionCommand::MakeStatic
            | ActionCommand::Release
    )
}

fn confirm_body(action_id: &str, label: &str, record_name: &str) -> String {
    match action_id {
        "reboot" => "Reboot the router? Active sessions will drop.".into(),
        "shutdown" => "Shut down the router? The device will power off.".into(),
        "backup-load" => {
            format!("Load backup {record_name}? This replaces running config and reboots.")
        }
        "make-static" => format!("Make lease {record_name} static?"),
        "upgrade" => "Upgrade RouterBOARD firmware? The router will reboot.".into(),
        "reset-configuration" => {
            "Reset configuration? This wipes the running config (keep-users if you set it).".into()
        }
        "flush" => format!("Flush {record_name}? Dynamic entries will be rebuilt."),
        "run" => format!("Run {record_name} now?"),
        "release" => format!("Release lease {record_name}?"),
        _ => format!("{label} {record_name}?"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppCommand, Screen};
    use crate::event::AppEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use mtui_core::ActionCommand;
    use mtui_routeros::{MASKED_VALUE, Resource};

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn enter_on_content_opens_edit_not_nav() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "vlan10".into());
        fields.insert("vlan-id".into(), "10".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "vlan".into(),
            rows: vec![Resource {
                id: "*3".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(matches!(app.overlay, Overlay::Form(_)));
    }

    fn open_vlan_editor() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "vlan10".into());
        fields.insert("vlan-id".into(), "10".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "vlan".into(),
            rows: vec![Resource {
                id: "*3".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        app
    }

    #[test]
    fn arrows_move_fields_and_tabs_from_a_text_input() {
        let mut app = open_vlan_editor();
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.section, 0);
        assert_eq!(session.focus, 0);

        let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.focus, 1);
        let _ = app.update(AppEvent::Input(press(KeyCode::Up)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.focus, 0);
        let _ = app.update(AppEvent::Input(press(KeyCode::Up)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.focus, 0);

        let _ = app.update(AppEvent::Input(press(KeyCode::Right)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.section, 1);
        let _ = app.update(AppEvent::Input(press(KeyCode::Left)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.section, 0);
        let _ = app.update(AppEvent::Input(press(KeyCode::Left)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.section, 0);
    }

    #[test]
    fn digits_type_into_text_and_number_fields_instead_of_jumping_tabs() {
        let mut app = open_vlan_editor();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('2'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.section, 0);
        assert_eq!(
            session.values.get("name").map(String::as_str),
            Some("vlan102")
        );

        let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('2'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.section, 0);
        assert_eq!(
            session.values.get("vlan-id").map(String::as_str),
            Some("102")
        );
        let _ = app.update(AppEvent::Input(press(KeyCode::Char(']'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.section, 1);
    }

    #[test]
    fn digits_still_jump_tabs_from_a_toggle() {
        let mut app = open_vlan_editor();
        for _ in 0..4 {
            let _ = app.update(AppEvent::Input(press(KeyCode::Down)));
        }
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.focus, 4);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('2'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected form");
        };
        assert_eq!(session.section, 1);
        assert_eq!(session.focus, 0);
    }

    #[test]
    fn wireguard_enter_opens_edit_and_n_opens_create() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("wireguard");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "wg1".into());
        fields.insert("listen-port".into(), "13231".into());
        fields.insert("private-key".into(), "MARKER-SECRET".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "wireguard".into(),
            rows: vec![Resource {
                id: "*8".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected wireguard editor, got {:?}", app.overlay);
        };
        assert_eq!(session.resource_id, "wireguard");
        assert_eq!(session.mode, mtui_ui::FormMode::Edit);
        assert_eq!(
            session.values.get("private-key").map(String::as_str),
            Some(MASKED_VALUE)
        );
        let _ = app.update(AppEvent::Input(press(KeyCode::Esc)));
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('n'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected wireguard create, got {:?}", app.overlay);
        };
        assert_eq!(session.mode, mtui_ui::FormMode::Create);
    }

    #[test]
    fn wireguard_peer_add_opens_create() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("wireguard-peers");
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('n'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected peer create, got {:?}", app.overlay);
        };
        assert_eq!(session.resource_id, "wireguard-peers");
        assert_eq!(session.mode, mtui_ui::FormMode::Create);
    }

    #[test]
    fn add_from_nav_opens_create() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        app.pane = Pane::Nav;
        let hints = app.footer_action_hints();
        assert!(
            hints
                .iter()
                .any(|(key, label)| key == "n" && label == "Add"),
            "nav footer should offer add: {hints:?}"
        );
        assert!(
            !hints.iter().any(|(key, _)| key == "e"),
            "row edit should stay on the table pane: {hints:?}"
        );
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('n'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected create form, got {:?}", app.overlay);
        };
        assert_eq!(session.resource_id, "vlan");
        assert_eq!(session.mode, mtui_ui::FormMode::Create);
    }

    #[test]
    fn edit_from_nav_does_not_open_the_sheet() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "vlan10".into());
        fields.insert("vlan-id".into(), "10".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "vlan".into(),
            rows: vec![Resource {
                id: "*3".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Nav;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('e'))));
        assert!(matches!(app.overlay, Overlay::None));
    }

    #[test]
    fn logs_c_does_not_open_copy() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("logs");
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('c'))));
        assert!(matches!(app.overlay, Overlay::None));
    }

    fn certificates_loaded() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("certificates");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "web".into());
        fields.insert("common-name".into(), "web.example".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "certificates".into(),
            rows: vec![Resource {
                id: "*c1".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        app
    }

    fn command_fields(cmds: &[AppCommand]) -> (&str, &str, &BTreeMap<String, String>) {
        match cmds.first() {
            Some(AppCommand::Mutate {
                op:
                    MutationOp::Command {
                        endpoint,
                        command,
                        fields,
                    },
                ..
            }) => (endpoint.as_str(), command.as_str(), fields),
            other => panic!("expected command mutate, got {other:?}"),
        }
    }

    #[test]
    fn certificate_sign_prompt_saves_id_and_ca() {
        let mut app = certificates_loaded();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('g'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected sign prompt, got {:?}", app.overlay);
        };
        assert_eq!(session.prompt_command, Some("sign"));
        assert!(session.values.contains_key("ca"));
        assert_eq!(session.record_id, "*c1");
        if let Overlay::Form(session) = &mut app.overlay {
            session.values.insert("ca".into(), "root-ca".into());
        }
        let cmds = app.save_form();
        let (endpoint, command, fields) = command_fields(&cmds);
        assert_eq!(endpoint, "/rest/certificate");
        assert_eq!(command, "sign");
        assert_eq!(fields.get(".id").map(String::as_str), Some("*c1"));
        assert_eq!(fields.get("ca").map(String::as_str), Some("root-ca"));
    }

    #[test]
    fn certificate_import_does_not_need_a_row() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("certificates");
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected import prompt, got {:?}", app.overlay);
        };
        assert_eq!(session.prompt_command, Some("import"));
        assert!(session.record_id.is_empty());
        if let Overlay::Form(session) = &mut app.overlay {
            session.values.insert("file-name".into(), "web.p12".into());
            session
                .values
                .insert("passphrase".into(), MASKED_VALUE.into());
            session.values.insert("name".into(), "web".into());
        }
        let cmds = app.save_form();
        let (endpoint, command, fields) = command_fields(&cmds);
        assert_eq!(endpoint, "/rest/certificate");
        assert_eq!(command, "import");
        assert!(!fields.contains_key(".id"));
        assert_eq!(fields.get("file-name").map(String::as_str), Some("web.p12"));
        assert_eq!(fields.get("name").map(String::as_str), Some("web"));
        assert!(!fields.contains_key("passphrase"));
    }

    #[test]
    fn certificate_export_includes_file_name() {
        let mut app = certificates_loaded();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('w'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected export prompt, got {:?}", app.overlay);
        };
        assert_eq!(session.prompt_command, Some("export-certificate"));
        if let Overlay::Form(session) = &mut app.overlay {
            session
                .values
                .insert("file-name".into(), "web-export".into());
            session.values.insert("type".into(), "pkcs12".into());
            session
                .values
                .insert("export-passphrase".into(), "hunter2".into());
        }
        let cmds = app.save_form();
        let (endpoint, command, fields) = command_fields(&cmds);
        assert_eq!(endpoint, "/rest/certificate");
        assert_eq!(command, "export-certificate");
        assert_eq!(fields.get(".id").map(String::as_str), Some("*c1"));
        assert_eq!(
            fields.get("file-name").map(String::as_str),
            Some("web-export")
        );
        assert_eq!(fields.get("type").map(String::as_str), Some("pkcs12"));
        assert_eq!(
            fields.get("export-passphrase").map(String::as_str),
            Some("hunter2")
        );
    }

    #[test]
    fn vlan_copy_prompt_still_sends_new_name() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "vlan10".into());
        fields.insert("vlan-id".into(), "10".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "vlan".into(),
            rows: vec![Resource {
                id: "*3".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('c'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected copy prompt, got {:?}", app.overlay);
        };
        assert_eq!(session.prompt_command, Some("copy"));
        assert_eq!(
            session.values.get("new-name").map(String::as_str),
            Some("vlan10-copy")
        );
        let cmds = app.save_form();
        let (endpoint, command, fields) = command_fields(&cmds);
        assert_eq!(endpoint, "/rest/interface/vlan");
        assert_eq!(command, "copy");
        assert_eq!(fields.get(".id").map(String::as_str), Some("*3"));
        assert_eq!(
            fields.get("new-name").map(String::as_str),
            Some("vlan10-copy")
        );
    }

    #[test]
    fn stale_mutate_is_ignored() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        let previous = app.poll_generation;
        app.poll_generation = previous.wrapping_add(1);
        let cmds = app.apply_mutate_result(WorkerMsg::MutateResult {
            session: app.test_session(),
            request_id: 1,
            generation: previous,
            error: None,
        });
        assert!(cmds.is_empty());
        assert!(matches!(app.overlay, Overlay::None));
    }

    fn command_op(cmds: &[crate::app::AppCommand]) -> &MutationOp {
        cmds.iter()
            .find_map(|cmd| match cmd {
                crate::app::AppCommand::Mutate { op, .. } => Some(op),
                _ => None,
            })
            .expect("mutate command")
    }

    fn ctrl_s() -> KeyEvent {
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
    }

    fn is_mutate(cmd: &AppCommand) -> bool {
        matches!(cmd, AppCommand::Mutate { .. })
    }

    #[test]
    fn reboot_from_empty_resources_uses_system_endpoint() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("resources");
        app.pane = Pane::Content;
        assert!(app.table.selected_row().is_none());
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('b'))));
        let Overlay::Confirm(session) = &app.overlay else {
            panic!("expected reboot confirm, got {:?}", app.overlay);
        };
        assert_eq!(session.command, ActionCommand::Reboot);
        assert_eq!(session.endpoint, "/rest/system");
        assert!(session.record_id.is_empty());
        assert!(session.body.contains("Active sessions will drop"));
        assert!(!session.body.contains("Safe Mode cannot undo"));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('y'))));
        match command_op(&cmds) {
            MutationOp::Command {
                endpoint,
                command,
                fields,
            } => {
                assert_eq!(endpoint, "/rest/system");
                assert_eq!(command, "reboot");
                assert!(fields.is_empty());
            }
            other => panic!("unexpected op {other:?}"),
        }
    }

    #[test]
    fn reboot_confirm_warns_when_safe_mode_is_on() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("resources");
        app.pane = Pane::Content;
        app.safe_mode = mtui_core::SafeModeStatus {
            enabled: true,
            current: true,
            owner: "api".into(),
            user: "admin".into(),
        };
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('b'))));
        let Overlay::Confirm(session) = &app.overlay else {
            panic!("expected reboot confirm, got {:?}", app.overlay);
        };
        assert!(
            session.body.contains("Safe Mode cannot undo"),
            "{}",
            session.body
        );
    }

    #[test]
    fn shutdown_from_empty_resources_uses_system_endpoint() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("resources");
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('o'))));
        let Overlay::Confirm(session) = &app.overlay else {
            panic!("expected shutdown confirm, got {:?}", app.overlay);
        };
        assert_eq!(session.command, ActionCommand::Shutdown);
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        match command_op(&cmds) {
            MutationOp::Command {
                endpoint,
                command,
                fields,
            } => {
                assert_eq!(endpoint, "/rest/system");
                assert_eq!(command, "shutdown");
                assert!(fields.is_empty());
            }
            other => panic!("unexpected op {other:?}"),
        }
    }

    #[test]
    fn backup_save_prompt_includes_name() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("files");
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('b'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected backup save prompt, got {:?}", app.overlay);
        };
        assert_eq!(session.prompt_command, Some("save"));
        assert!(session.values.contains_key("name"));
        assert!(session.values.contains_key("password"));
        if let Overlay::Form(session) = &mut app.overlay {
            session.values.insert("name".into(), "nightly".into());
            session.values.insert("password".into(), "secret".into());
        }
        let cmds = app.save_form();
        match command_op(&cmds) {
            MutationOp::Command {
                endpoint,
                command,
                fields,
            } => {
                assert_eq!(endpoint, "/rest/system/backup");
                assert_eq!(command, "save");
                assert_eq!(fields.get("name").map(String::as_str), Some("nightly"));
                assert_eq!(fields.get("password").map(String::as_str), Some("secret"));
                assert!(!fields.contains_key(".id"));
            }
            other => panic!("unexpected op {other:?}"),
        }
    }

    fn load_files(app: &mut App, name: &str) {
        let mut fields = HashMap::new();
        fields.insert("name".into(), name.into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "files".into(),
            rows: vec![Resource {
                id: "*9".into(),
                fields,
            }],
            error: None,
        }));
    }

    fn files_app(contents: Option<&str>) -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("files");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "backup.rsc".into());
        fields.insert("type".into(), "script".into());
        if let Some(text) = contents {
            fields.insert("contents".into(), text.into());
        }
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "files".into(),
            rows: vec![Resource {
                id: "*1".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        app
    }

    #[test]
    fn backup_load_on_backup_file_posts_name() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("files");
        load_files(&mut app, "foo.backup");
        app.pane = Pane::Content;
        let ids: Vec<_> = app
            .current_actions()
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(ids.contains(&"backup-load"));
        let cmds = app.dispatch_named_action("backup-load");
        assert!(cmds.is_empty());
        let Overlay::Confirm(session) = &app.overlay else {
            panic!("expected load confirm, got {:?}", app.overlay);
        };
        assert_eq!(
            session.fields.get("name").map(String::as_str),
            Some("foo.backup")
        );
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('y'))));
        match command_op(&cmds) {
            MutationOp::Command {
                endpoint,
                command,
                fields,
            } => {
                assert_eq!(endpoint, "/rest/system/backup");
                assert_eq!(command, "load");
                assert_eq!(fields.get("name").map(String::as_str), Some("foo.backup"));
            }
            other => panic!("unexpected op {other:?}"),
        }
    }

    #[test]
    fn backup_load_not_offered_for_other_files() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("files");
        load_files(&mut app, "script.rsc");
        app.pane = Pane::Content;
        let ids: Vec<_> = app
            .current_actions()
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(!ids.contains(&"backup-load"));
        let cmds = app.dispatch_named_action("backup-load");
        assert!(cmds.is_empty());
        assert!(matches!(app.overlay, Overlay::None));
    }

    #[test]
    fn enable_confirm_still_requires_a_row() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "vlan10".into());
        fields.insert("disabled".into(), "true".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "vlan".into(),
            rows: vec![Resource {
                id: "*3".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('d'))));
        let Overlay::Confirm(session) = &app.overlay else {
            panic!("expected enable confirm, got {:?}", app.overlay);
        };
        assert_eq!(session.command, ActionCommand::Enable);
        assert_eq!(session.record_id, "*3");
        assert_eq!(session.endpoint, "/rest/interface/vlan");
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('y'))));
        match command_op(&cmds) {
            MutationOp::Command {
                endpoint,
                command,
                fields,
            } => {
                assert_eq!(endpoint, "/rest/interface/vlan");
                assert_eq!(command, "enable");
                assert_eq!(fields.get(".id").map(String::as_str), Some("*3"));
            }
            other => panic!("unexpected op {other:?}"),
        }
    }

    fn load_named_rows(resource_id: &str, rows: Vec<Resource>) -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource(resource_id);
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: resource_id.into(),
            rows,
            error: None,
        }));
        app.pane = Pane::Content;
        app
    }

    fn filter_rule(id: &str, comment: &str) -> Resource {
        let mut fields = HashMap::new();
        fields.insert("chain".into(), "forward".into());
        fields.insert("action".into(), "accept".into());
        fields.insert("comment".into(), comment.into());
        Resource {
            id: id.into(),
            fields,
        }
    }

    #[test]
    fn move_up_on_middle_row_posts_previous_id() {
        let mut app = load_named_rows(
            "firewall-filter",
            vec![
                filter_rule("*1", "first"),
                filter_rule("*2", "middle"),
                filter_rule("*3", "last"),
            ],
        );
        app.table.selected = 1;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('['))));
        assert!(matches!(app.overlay, Overlay::None));
        let Some(AppCommand::Mutate { op, .. }) = cmds.into_iter().next() else {
            panic!("expected mutate command");
        };
        assert_eq!(
            op,
            MutationOp::Command {
                endpoint: "/rest/ip/firewall/filter".into(),
                command: "move".into(),
                fields: BTreeMap::from([
                    (".id".into(), "*2".into()),
                    ("destination".into(), "*1".into()),
                ]),
            }
        );
    }

    #[test]
    fn move_up_on_first_row_is_a_noop() {
        let mut app = load_named_rows(
            "firewall-filter",
            vec![filter_rule("*1", "first"), filter_rule("*2", "second")],
        );
        app.table.selected = 0;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('['))));
        assert!(cmds.is_empty());
        assert!(matches!(app.overlay, Overlay::None));
        assert_eq!(app.status, "Already first");
    }

    #[test]
    fn make_static_confirms_then_posts_command() {
        let mut fields = HashMap::new();
        fields.insert("address".into(), "192.168.88.10".into());
        fields.insert("mac-address".into(), "00:11:22:33:44:55".into());
        let mut app = load_named_rows(
            "dhcp-leases",
            vec![Resource {
                id: "*9".into(),
                fields,
            }],
        );
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('m'))));
        assert!(cmds.is_empty());
        let Overlay::Confirm(session) = &app.overlay else {
            panic!("expected confirm, got {:?}", app.overlay);
        };
        assert_eq!(session.body, "Make lease 192.168.88.10 static?");
        assert_eq!(session.command, ActionCommand::MakeStatic);
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('y'))));
        let Some(AppCommand::Mutate { op, .. }) = cmds.into_iter().next() else {
            panic!("expected mutate command");
        };
        assert_eq!(
            op,
            MutationOp::Command {
                endpoint: "/rest/ip/dhcp-server/lease".into(),
                command: "make-static".into(),
                fields: BTreeMap::from([(".id".into(), "*9".into())]),
            }
        );
    }

    #[test]
    fn files_remove_and_fetch_keys_are_offered() {
        let app = files_app(None);
        let ids: Vec<_> = app
            .current_actions()
            .iter()
            .map(|action| action.id)
            .collect();
        assert!(ids.contains(&"remove"));
        assert!(ids.contains(&"fetch"));
        assert!(!ids.contains(&"upload"));
        assert!(!ids.contains(&"download"));
        let hints = app.footer_action_hints();
        assert!(
            hints
                .iter()
                .any(|(key, label)| key == "f" && label == "Fetch URL")
        );
        assert!(
            hints
                .iter()
                .any(|(key, label)| key == "x" && label == "Remove")
        );
    }

    #[test]
    fn fetch_prompt_requires_url() {
        let mut app = files_app(None);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('f'))));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected fetch form, got {:?}", app.overlay);
        };
        assert_eq!(session.prompt_command, Some("fetch"));
        let cmds = app.update(AppEvent::Input(ctrl_s()));
        assert!(cmds.iter().all(|cmd| !is_mutate(cmd)));
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected fetch form");
        };
        assert_eq!(session.error.as_deref(), Some("URL is required"));
    }

    fn ping_screen() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("ping");
        app.pane = Pane::Content;
        app
    }

    fn traceroute_screen() -> App {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("traceroute");
        app.pane = Pane::Content;
        app
    }

    fn type_chars(app: &mut App, text: &str) {
        for ch in text.chars() {
            let _ = app.update(AppEvent::Input(press(KeyCode::Char(ch))));
        }
    }

    #[test]
    fn opening_ping_overlay_does_not_fetch_identity() {
        let mut app = ping_screen();
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        assert!(cmds.is_empty(), "overlay open must not issue I/O: {cmds:?}");
        assert!(matches!(app.overlay, Overlay::Probe(_)));
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("expected ping overlay");
        };
        assert_eq!(probe.kind, ProbeKind::Ping);
    }

    #[test]
    fn ping_local_fetch_is_empty_without_error() {
        let mut app = ping_screen();
        let cmds = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "ping".into(),
            rows: Vec::new(),
            error: None,
        }));
        assert!(cmds.is_empty());
        assert!(!app.loading);
        assert!(!app.status.contains("Refresh failed"));
        assert!(app.table.rows.is_empty());
    }

    #[test]
    fn starting_ping_emits_fetch_ping_with_address() {
        let mut app = ping_screen();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        type_chars(&mut app, "192.0.2.1");
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        match cmds.as_slice() {
            [
                AppCommand::FetchPing {
                    address,
                    count,
                    src,
                    ..
                },
            ] => {
                assert_eq!(address, "192.0.2.1");
                assert_eq!(count, "4");
                assert!(src.is_empty());
            }
            other => panic!("expected FetchPing, got {other:?}"),
        }
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("overlay stays open");
        };
        assert!(probe.running);
        assert!(probe.samples.is_empty());
    }

    #[test]
    fn empty_ping_address_shows_status_error() {
        let mut app = ping_screen();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(cmds.is_empty());
        assert_eq!(app.status, "Address is required");
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("expected ping overlay");
        };
        assert!(!probe.running);
    }

    #[test]
    fn stale_ping_generation_is_ignored() {
        let mut app = ping_screen();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        type_chars(&mut app, "192.0.2.1");
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("expected ping overlay");
        };
        let stale = probe.generation.wrapping_sub(1);
        let cmds = app.update(AppEvent::Worker(WorkerMsg::PingTraceResult {
            session: app.test_session(),
            generation: stale,
            rows: vec![HashMap::from([("host".into(), "stale".into())])],
            error: None,
            done: false,
        }));
        assert!(cmds.is_empty());
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("expected ping overlay");
        };
        assert!(probe.samples.is_empty());
        assert!(probe.running);
    }

    #[test]
    fn traceroute_enter_opens_overlay_and_start_fetches() {
        let mut app = traceroute_screen();
        let open = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(open.is_empty());
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("expected traceroute overlay, got {:?}", app.overlay);
        };
        assert_eq!(probe.kind, ProbeKind::Traceroute);
        type_chars(&mut app, "192.0.2.1");
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        match cmds.as_slice() {
            [
                AppCommand::FetchTraceroute {
                    address,
                    count,
                    protocol,
                    ..
                },
            ] => {
                assert_eq!(address, "192.0.2.1");
                assert_eq!(count, "8");
                assert_eq!(protocol, "icmp");
            }
            other => panic!("expected FetchTraceroute, got {other:?}"),
        }
    }

    #[test]
    fn empty_traceroute_address_shows_status_error() {
        let mut app = traceroute_screen();
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(cmds.is_empty());
        assert_eq!(app.status, "Address is required");
    }

    #[test]
    fn ping_keeps_samples_while_running() {
        let mut app = ping_screen();
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        type_chars(&mut app, "192.0.2.1");
        let first = app.update(AppEvent::Input(press(KeyCode::Enter)));
        let AppCommand::FetchPing { generation, .. } = first[0] else {
            panic!("expected FetchPing");
        };
        let _ = app.update(AppEvent::Worker(WorkerMsg::PingTraceResult {
            session: app.test_session(),
            generation,
            rows: vec![HashMap::from([("host".into(), "192.0.2.1".into())])],
            error: None,
            done: true,
        }));
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("expected ping overlay");
        };
        assert_eq!(probe.samples.len(), 1);
        assert!(!probe.running);
        let second = app.update(AppEvent::Input(press(KeyCode::Enter)));
        assert!(matches!(second.as_slice(), [AppCommand::FetchPing { .. }]));
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("expected ping overlay");
        };
        assert_eq!(probe.samples.len(), 1);
        assert!(probe.running);
    }

    #[test]
    fn bandwidth_test_overlay_emits_fetch_probe() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("bandwidth-test");
        app.pane = Pane::Content;
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        assert!(cmds.is_empty(), "overlay open must not issue I/O: {cmds:?}");
        let Overlay::Probe(probe) = &app.overlay else {
            panic!("expected bandwidth overlay, got {:?}", app.overlay);
        };
        assert_eq!(probe.kind, ProbeKind::BandwidthTest);
        type_chars(&mut app, "192.0.2.8");
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        match cmds.as_slice() {
            [
                AppCommand::FetchProbe {
                    endpoint,
                    command,
                    fields,
                    ..
                },
            ] => {
                assert_eq!(endpoint, "/rest/tool");
                assert_eq!(command, "bandwidth-test");
                assert_eq!(fields.get("address").map(String::as_str), Some("192.0.2.8"));
                assert_eq!(fields.get("duration").map(String::as_str), Some("10"));
                assert_eq!(fields.get("protocol").map(String::as_str), Some("tcp"));
            }
            other => panic!("expected FetchProbe, got {other:?}"),
        }
    }

    #[test]
    fn profiler_starts_without_address() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("profiler");
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Enter)));
        match cmds.as_slice() {
            [
                AppCommand::FetchProbe {
                    command, fields, ..
                },
            ] => {
                assert_eq!(command, "profile");
                assert_eq!(fields.get("duration").map(String::as_str), Some("5"));
                assert!(!fields.contains_key("address"));
            }
            other => panic!("expected FetchProbe, got {other:?}"),
        }
    }

    #[test]
    fn wol_prompt_posts_tool_wol() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("wol");
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('p'))));
        let Overlay::Form(session) = &mut app.overlay else {
            panic!("expected wol prompt, got {:?}", app.overlay);
        };
        session.values.insert("interface".into(), "ether1".into());
        session
            .values
            .insert("mac".into(), "4C:5E:0C:00:00:01".into());
        let cmds = app.save_form();
        let (endpoint, command, fields) = command_fields(&cmds);
        assert_eq!(endpoint, "/rest/tool");
        assert_eq!(command, "wol");
        assert_eq!(fields.get("interface").map(String::as_str), Some("ether1"));
        assert_eq!(
            fields.get("mac").map(String::as_str),
            Some("4C:5E:0C:00:00:01")
        );
        assert!(!fields.contains_key(".id"));
    }

    #[test]
    fn dns_cache_flush_does_not_attach_selected_id() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("dns-cache");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "example.com".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "dns-cache".into(),
            rows: vec![Resource {
                id: "*9".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('f'))));
        let Overlay::Confirm(session) = &app.overlay else {
            panic!("expected flush confirm, got {:?}", app.overlay);
        };
        assert_eq!(session.command, ActionCommand::Flush);
        assert!(session.record_id.is_empty());
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('y'))));
        match command_op(&cmds) {
            MutationOp::Command {
                command, fields, ..
            } => {
                assert_eq!(command, "flush");
                assert!(!fields.contains_key(".id"));
            }
            other => panic!("unexpected op {other:?}"),
        }
    }

    #[test]
    fn ctrl_s_previews_changed_fields_before_patch() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        let mut fields = HashMap::new();
        fields.insert("name".into(), "vlan10".into());
        fields.insert("vlan-id".into(), "10".into());
        fields.insert("comment".into(), "old".into());
        let _ = app.update(AppEvent::Worker(WorkerMsg::ResourceResult {
            session: app.test_session(),
            request_id: app.request_id,
            generation: app.poll_generation,
            resource_id: "vlan".into(),
            rows: vec![Resource {
                id: "*4".into(),
                fields,
            }],
            error: None,
        }));
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Enter)));
        if let Overlay::Form(session) = &mut app.overlay {
            session.values.insert("comment".into(), "office".into());
        } else {
            panic!("expected form");
        }
        let cmds = app.save_form();
        assert!(cmds.is_empty());
        let Overlay::Form(session) = &app.overlay else {
            panic!("expected preview, got {:?}", app.overlay);
        };
        assert!(session.confirm_save);
        let cmds = app.save_form();
        match command_op(&cmds) {
            MutationOp::Patch { id, fields, .. } => {
                assert_eq!(id.as_deref(), Some("*4"));
                assert_eq!(fields.get("comment").map(String::as_str), Some("office"));
                assert!(!fields.contains_key("name"));
            }
            other => panic!("unexpected op {other:?}"),
        }
    }

    #[test]
    fn bulk_disable_confirms_all_checked_firewall_rows() {
        let mut app = load_named_rows(
            "firewall-filter",
            vec![filter_rule("*1", "first"), filter_rule("*2", "second")],
        );
        app.pane = Pane::Content;
        let _ = app.update(AppEvent::Input(press(KeyCode::Char(' '))));
        app.table.move_selection(1);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char(' '))));
        assert_eq!(app.table.checked_count(), 2);
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('d'))));
        let Overlay::Confirm(session) = &app.overlay else {
            panic!("expected bulk confirm, got {:?}", app.overlay);
        };
        assert_eq!(session.record_ids.len(), 2);
        assert!(session.record_name.contains("2 items"));
        let cmds = app.update(AppEvent::Input(press(KeyCode::Char('y'))));
        match command_op(&cmds) {
            MutationOp::Batch { ops } => assert_eq!(ops.len(), 2),
            other => panic!("expected batch, got {other:?}"),
        }
    }
}
