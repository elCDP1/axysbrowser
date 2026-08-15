use gtk::Widget;
use gtk::glib::object::Cast;
use std::rc::Rc;

use crate::app_state::AppState;

use super::pages::{
    about::build_about, downloads::build_downloads, newtab::build_newtab, privacy::build_privacy,
    settings::build_settings, tools::build_tools, welcome::build_welcome,
};

pub fn page_name(uri: &str) -> Option<&'static str> {
    match uri {
        "axys://" | "axys://newtab" => Some("newtab"),

        "axys://welcome" => Some("welcome"),

        "axys://privacy" => Some("privacy"),

        "axys://about" => Some("about"),

        "axys://settings" => Some("settings"),

        "axys://tools" => Some("tools"),

        "axys://downloads" => Some("downloads"),

        _ => None,
    }
}

/// Display title for an internal `axys://` page, used as the tab title
/// since these pages aren't loaded in the `WebView` and never trigger its
/// `title-notify` signal.
pub fn page_title(uri: &str) -> &'static str {
    match uri {
        "axys://welcome" => "Welcome",

        "axys://privacy" => "Privacy",

        "axys://about" => "About axysBrowser",

        "axys://settings" => "Settings",

        "axys://tools" => "Tools",

        "axys://downloads" => "Downloads",

        _ => "New Tab",
    }
}

pub fn route(
    uri: &str,
    on_search: Rc<dyn Fn(String)>,
    on_about: Rc<dyn Fn()>,
    on_extensions_changed: Rc<dyn Fn(bool)>,
    on_downloads: Rc<dyn Fn()>,
    state: Rc<AppState>,
) -> Option<Widget> {
    match uri {
        "axys://" | "axys://newtab" => Some(build_newtab(on_search, on_about).upcast()),

        "axys://welcome" => Some(build_welcome().upcast()),

        "axys://privacy" => Some(build_privacy().upcast()),

        "axys://about" => Some(build_about().upcast()),

        "axys://settings" => Some(build_settings(state, on_extensions_changed).upcast()),

        "axys://tools" => Some(build_tools(on_downloads).upcast()),

        "axys://downloads" => Some(build_downloads(state).upcast()),

        _ => None,
    }
}
