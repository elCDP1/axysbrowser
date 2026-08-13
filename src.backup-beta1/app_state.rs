use crate::browser::address::{SearchConfig, load_config};
use gtk::Settings as GtkSettings;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use webkit6::NetworkSession;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSettings {
    pub dark_mode: bool,
    pub search_engine: String,
    pub show_extensions: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            search_engine: "brave".to_string(),
            show_extensions: true,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Rc<RefCell<UserSettings>>,
    pub search: Rc<RefCell<SearchConfig>>,
    pub network_session: NetworkSession,
}

impl AppState {
    pub fn load() -> Rc<Self> {
        let mut settings = Self::load_user_settings();
        let mut search = load_config();

        if !search.engines.contains_key(&settings.search_engine) {
            settings.search_engine = search.default_engine.clone();
        }

        search.default_engine = settings.search_engine.clone();

        let network_session = NetworkSession::new(None, None);

        network_session.set_persistent_credential_storage_enabled(true);

        let state = Rc::new(Self {
            settings: Rc::new(RefCell::new(settings)),
            search: Rc::new(RefCell::new(search)),
            network_session,
        });

        state.apply_theme();

        state
    }

    fn config_path() -> PathBuf {
        if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("axysbrowser").join("settings.toml");
        }

        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home)
                .join(".config")
                .join("axysbrowser")
                .join("settings.toml");
        }

        PathBuf::from("settings.toml")
    }

    fn load_user_settings() -> UserSettings {
        let path = Self::config_path();

        let Ok(text) = fs::read_to_string(path) else {
            return UserSettings::default();
        };

        toml::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let settings = self.settings.borrow();

        if let Ok(text) = toml::to_string_pretty(&*settings) {
            let _ = fs::write(path, text);
        }
    }

    pub fn apply_theme(&self) {
        let dark = self.settings.borrow().dark_mode;

        if let Some(gtk_settings) = GtkSettings::default() {
            gtk_settings.set_property("gtk-application-prefer-dark-theme", dark);
        }
    }

    pub fn set_search_engine(&self, engine: &str) {
        if !self.search.borrow().engines.contains_key(engine) {
            return;
        }

        self.search.borrow_mut().default_engine = engine.to_string();

        self.settings.borrow_mut().search_engine = engine.to_string();

        self.save();
    }

    pub fn set_dark_mode(&self, dark: bool) {
        self.settings.borrow_mut().dark_mode = dark;

        self.apply_theme();
        self.save();
    }

    pub fn set_extensions_visible(&self, visible: bool) {
        self.settings.borrow_mut().show_extensions = visible;

        self.save();
    }
}
