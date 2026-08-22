//! Login form state.

/// True for runes that may be typed into a text field (not control/modifier noise).
#[must_use]
pub fn is_printable_char(ch: char) -> bool {
    !ch.is_control()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    Url,
    Username,
    Password,
}

impl LoginField {
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Url => Self::Username,
            Self::Username => Self::Password,
            Self::Password => Self::Url,
        }
    }

    #[must_use]
    pub fn prev(self) -> Self {
        match self {
            Self::Url => Self::Password,
            Self::Username => Self::Url,
            Self::Password => Self::Username,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoginForm {
    pub url: String,
    pub username: String,
    pub password: String,
    pub focus: LoginField,
    pub error: Option<String>,
}

impl Default for LoginForm {
    fn default() -> Self {
        Self {
            url: String::from("https://"),
            username: String::new(),
            password: String::new(),
            focus: LoginField::Url,
            error: None,
        }
    }
}

impl LoginForm {
    pub fn focused_mut(&mut self) -> &mut String {
        match self.focus {
            LoginField::Url => &mut self.url,
            LoginField::Username => &mut self.username,
            LoginField::Password => &mut self.password,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        if !is_printable_char(ch) {
            return;
        }
        self.focused_mut().push(ch);
    }

    pub fn backspace(&mut self) {
        self.focused_mut().pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_char_appends_printable_runes() {
        let mut form = LoginForm {
            focus: LoginField::Username,
            ..LoginForm::default()
        };
        form.insert_char('a');
        form.insert_char('d');
        form.insert_char('m');
        form.insert_char('i');
        form.insert_char('n');
        assert_eq!(form.username, "admin");
    }

    #[test]
    fn insert_char_ignores_control_runes() {
        let mut form = LoginForm {
            focus: LoginField::Password,
            ..LoginForm::default()
        };
        form.insert_char('\0');
        form.insert_char('\r');
        form.insert_char('\n');
        form.insert_char('\u{1b}');
        form.insert_char('\u{8}');
        assert_eq!(form.password, "");
        form.insert_char('P');
        form.insert_char('ä');
        assert_eq!(form.password, "Pä");
    }

    #[test]
    fn backspace_removes_last_character_of_focused_field() {
        let mut form = LoginForm {
            url: "https://router".into(),
            ..LoginForm::default()
        };
        form.backspace();
        assert_eq!(form.url, "https://route");
        form.focus = LoginField::Username;
        form.username = "admin".into();
        form.backspace();
        assert_eq!(form.username, "admi");
        assert_eq!(form.url, "https://route");
    }

    #[test]
    fn backspace_on_empty_field_is_a_no_op() {
        let mut form = LoginForm {
            focus: LoginField::Username,
            ..LoginForm::default()
        };
        form.backspace();
        assert_eq!(form.username, "");
    }

    #[test]
    fn backspace_pops_a_whole_unicode_scalar() {
        let mut form = LoginForm {
            focus: LoginField::Password,
            password: "Päss".into(),
            ..LoginForm::default()
        };
        form.backspace();
        assert_eq!(form.password, "Päs");
    }
}
