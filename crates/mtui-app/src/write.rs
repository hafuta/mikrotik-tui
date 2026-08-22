//! Resource write overlays and mutation commands.

use std::collections::{BTreeMap, HashMap};

use mtui_core::{
    ActionCommand, ActionKind, ActionSpec, DASHBOARD_ID, INTERFACE_CREATE_TARGETS, action_label,
    patch_body, resource_by_id, truthy,
};
use mtui_routeros::MASKED_VALUE;
use mtui_ui::{ActionMenuItem, ActionMenuState, FormSession, Row, TorchState};

use crate::app::{App, AppCommand, Overlay, Pane};
use crate::event::WorkerMsg;

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmSession {
    pub title: String,
    pub body: String,
    pub action_id: String,
    pub command: ActionCommand,
    pub record_id: String,
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
        if self.resource_actions_allowed() {
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
        hints.push(("r".into(), "refresh".into()));
        hints.push(("q".into(), "quit".into()));
        hints
    }

    fn resource_actions_allowed(&self) -> bool {
        self.current_resource != "logs" && self.current_resource != DASHBOARD_ID
    }

    fn pane_allows_row_actions(&self) -> bool {
        matches!(self.pane, Pane::Content | Pane::Inspector)
    }

    fn action_offered_in_pane(&self, action: &ActionSpec) -> bool {
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
        match action.kind {
            ActionKind::Edit => self.open_edit(),
            ActionKind::Create => self.open_create(&self.current_resource.clone()),
            ActionKind::Confirm { command } => self.open_confirm(action, command),
            ActionKind::Prompt { command } => match command {
                ActionCommand::BackupSave => self.open_backup_save_prompt(),
                _ => self.open_copy_prompt(command),
            },
            ActionKind::Overlay { id: "torch" } => self.open_torch(),
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

    fn open_copy_prompt(&mut self, command: ActionCommand) -> Vec<AppCommand> {
        let Some(row) = self.table.selected_row() else {
            return Vec::new();
        };
        let id = row.get(".id").cloned().unwrap_or_default();
        let name = row
            .get("name")
            .or_else(|| row.get("interface"))
            .cloned()
            .unwrap_or_else(|| id.clone());
        self.overlay = Overlay::Form(FormSession::prompt(
            self.current_resource.clone(),
            id,
            command.rest_name(),
            &name,
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
        let mut record_id = row
            .and_then(|row| row.get(".id"))
            .cloned()
            .unwrap_or_default();
        let record_name = row
            .and_then(|row| row.get("name").or_else(|| row.get("interface")))
            .cloned()
            .unwrap_or_else(|| record_id.clone());
        if matches!(action.id, "reboot" | "shutdown" | "backup-load") {
            record_id.clear();
        }
        let command = match (command, row) {
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
        let body = confirm_body(action.id, &label, &record_name);
        self.overlay = Overlay::Confirm(ConfirmSession {
            title: label,
            body,
            action_id: action.id.to_string(),
            command,
            record_id,
            record_name,
            endpoint: command_base_path(action.id, spec.endpoint()),
            fields,
        });
        tracing::trace!(overlay = "confirm", action = action.id, "opened pane");
        Vec::new()
    }

    fn open_action_menu(&mut self) -> Vec<AppCommand> {
        let row = self.table.selected_row();
        let items: Vec<ActionMenuItem> = self
            .current_actions()
            .into_iter()
            .map(|action| ActionMenuItem {
                id: action.id.to_string(),
                label: action_label(action, row),
                keys: action.key.map_or_else(String::new, |key| key.to_string()),
                danger: action.danger,
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
        } else {
            let mut fields = BTreeMap::new();
            fields.insert(".id".into(), record_id);
            if let Some(name) = values.get("new-name") {
                fields.insert("new-name".into(), name.clone());
            }
            (spec.endpoint().to_string(), fields, "Copying…")
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
        let Overlay::Form(session) = &self.overlay else {
            return Vec::new();
        };
        if session.saving {
            return Vec::new();
        }
        if let Some(command) = session.prompt_command {
            return self.save_prompt_form(command);
        }
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

    pub(crate) fn confirm_pending(&mut self) -> Vec<AppCommand> {
        let Overlay::Confirm(session) = &self.overlay else {
            return Vec::new();
        };
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

    pub(crate) fn mutate_command(&mut self, op: MutationOp) -> AppCommand {
        AppCommand::Mutate {
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
        if let Some(err) = error {
            if let Overlay::Form(session) = &mut self.overlay {
                session.saving = false;
                session.error = Some(err.clone());
            }
            self.status = format!("Write failed: {err}");
            return Vec::new();
        }
        self.overlay = Overlay::None;
        self.status = "Saved".into();
        self.refreshing = true;
        self.poll_current()
    }

    pub(crate) fn apply_torch_result(
        &mut self,
        generation: u64,
        rows: Vec<HashMap<String, String>>,
        error: Option<String>,
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
        if torch.running {
            self.torch_sample_command()
        } else {
            Vec::new()
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

pub(crate) fn json_rows(value: serde_json::Value) -> Vec<HashMap<String, String>> {
    match value {
        serde_json::Value::Array(items) => items.into_iter().filter_map(object_row).collect(),
        serde_json::Value::Object(_) => object_row(value).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn object_row(value: serde_json::Value) -> Option<HashMap<String, String>> {
    let serde_json::Value::Object(map) = value else {
        return None;
    };
    let mut row = HashMap::new();
    for (key, value) in map {
        let text = match value {
            serde_json::Value::String(text) => text,
            serde_json::Value::Number(number) => number.to_string(),
            serde_json::Value::Bool(flag) => flag.to_string(),
            serde_json::Value::Null => continue,
            other => other.to_string(),
        };
        row.insert(key, text);
    }
    Some(row)
}

fn command_base_path(action_id: &str, resource_endpoint: &str) -> String {
    match action_id {
        "reboot" | "shutdown" => "/rest/system".into(),
        "backup-save" | "backup-load" => "/rest/system/backup".into(),
        _ => resource_endpoint.to_string(),
    }
}

fn confirm_body(action_id: &str, label: &str, record_name: &str) -> String {
    match action_id {
        "reboot" => "Reboot the router? Active sessions will drop.".into(),
        "shutdown" => "Shut down the router? The device will power off.".into(),
        "backup-load" => {
            format!("Load backup {record_name}? This replaces running config and reboots.")
        }
        _ => format!("{label} {record_name}?"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Screen;
    use crate::event::AppEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

    #[test]
    fn stale_mutate_is_ignored() {
        let mut app = App::new(false).expect("app");
        app.screen = Screen::Main;
        app.select_resource("vlan");
        let previous = app.poll_generation;
        app.poll_generation = previous.wrapping_add(1);
        let cmds = app.apply_mutate_result(WorkerMsg::MutateResult {
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
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('u'))));
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
        let _ = app.update(AppEvent::Input(press(KeyCode::Char('u'))));
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
}
