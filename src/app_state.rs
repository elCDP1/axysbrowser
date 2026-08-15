use crate::browser::address::{SearchConfig, load_config};
use crate::browser::downloads::DownloadManager;
use crate::i18n;
use crate::theme;
use gtk::Application;
use gtk::CssProvider;
use gtk::Settings as GtkSettings;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use webkit6::NetworkSession;

fn default_true() -> bool {
    true
}

fn default_search_engine() -> String {
    "brave".to_string()
}

fn default_language() -> String {
    i18n::system_locale().to_string()
}

type LanguageListener = Weak<dyn Fn()>;
type LanguageListeners = Rc<RefCell<Vec<LanguageListener>>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSettings {
    #[serde(default = "default_true")]
    pub dark_mode: bool,

    #[serde(default = "default_search_engine")]
    pub search_engine: String,

    #[serde(default = "default_true")]
    pub show_extensions: bool,

    /// Enables the WebKit Web Inspector for tabs opened after this is toggled.
    /// Does not retroactively affect tabs that are already open.
    #[serde(default = "default_true")]
    pub developer_tools: bool,

    /// Enables WebKit's Intelligent Tracking Prevention on the shared,
    /// persistent network session used by normal (non-private) windows.
    /// Applies immediately, including to already-open tabs.
    #[serde(default = "default_true")]
    pub tracking_prevention: bool,

    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            dark_mode: true,
            search_engine: default_search_engine(),
            show_extensions: true,
            developer_tools: true,
            tracking_prevention: true,
            language: default_language(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Rc<RefCell<UserSettings>>,
    pub search: Rc<RefCell<SearchConfig>>,
    pub network_session: NetworkSession,
    pub css_provider: CssProvider,
    pub css_installed: Rc<Cell<bool>>,
    pub downloads: DownloadManager,
    language_listeners: LanguageListeners,
}

impl AppState {
    pub fn load(application: &Application) -> Rc<Self> {
        let mut settings = Self::load_user_settings();

        settings.language = i18n::normalize_locale(&settings.language).to_string();

        i18n::set_locale(&settings.language);

        let mut search = load_config();

        if !search.engines.contains_key(&settings.search_engine) {
            settings.search_engine = search.default_engine.clone();
        }

        search.default_engine = settings.search_engine.clone();

        let network_session = NetworkSession::new(None, None);

        network_session.set_persistent_credential_storage_enabled(true);

        network_session.set_itp_enabled(settings.tracking_prevention);

        let downloads = DownloadManager::new(application.clone());

        downloads.watch(&network_session);

        let state = Rc::new(Self {
            settings: Rc::new(RefCell::new(settings)),
            search: Rc::new(RefCell::new(search)),
            network_session,
            css_provider: CssProvider::new(),
            css_installed: Rc::new(Cell::new(false)),
            downloads,
            language_listeners: Rc::new(RefCell::new(Vec::new())),
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

        let mut settings: UserSettings = toml::from_str(&text).unwrap_or_default();

        settings.language = i18n::normalize_locale(&settings.language).to_string();

        settings
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

        self.css_provider.load_from_data(&theme::stylesheet(dark));
    }

    pub fn subscribe_language(&self, callback: &Rc<dyn Fn()>) {
        self.language_listeners
            .borrow_mut()
            .push(Rc::downgrade(callback));
    }

    fn notify_language_changed(&self) {
        let mut listeners = self.language_listeners.borrow_mut();

        listeners.retain(|listener| listener.strong_count() > 0);

        let active_listeners = listeners.clone();

        drop(listeners);

        for listener in active_listeners {
            if let Some(callback) = listener.upgrade() {
                callback();
            }
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

    pub fn set_developer_tools(&self, enabled: bool) {
        self.settings.borrow_mut().developer_tools = enabled;

        self.save();
    }

    pub fn set_tracking_prevention(&self, enabled: bool) {
        self.settings.borrow_mut().tracking_prevention = enabled;

        self.network_session.set_itp_enabled(enabled);

        self.save();
    }

    pub fn set_language(&self, language: &str) {
        let language = i18n::normalize_locale(language);

        if self.settings.borrow().language == language {
            return;
        }

        i18n::set_locale(language);

        self.settings.borrow_mut().language = language.to_string();

        self.save();

        self.notify_language_changed();
    }
}
