use gtk::Widget;
use gtk::glib::object::Cast;
use std::rc::Rc;

use crate::app_state::AppState;

use super::pages::{
    about::build_about, newtab::build_newtab, privacy::build_privacy, settings::build_settings,
    tools::build_tools, welcome::build_welcome,
};

pub fn page_name(uri: &str) -> Option<&'static str> {
    match uri {
        "axys://" | "axys://newtab" => Some("newtab"),

        "axys://welcome" => Some("welcome"),

        "axys://privacy" => Some("privacy"),

        "axys://about" => Some("about"),

        "axys://settings" => Some("settings"),

        "axys://tools" => Some("tools"),

        _ => None,
    }
}

pub fn route(
    uri: &str,
    on_search: Rc<dyn Fn(String)>,
    on_about: Rc<dyn Fn()>,
    on_extensions_changed: Rc<dyn Fn(bool)>,
    state: Rc<AppState>,
) -> Option<Widget> {
    match uri {
        "axys://" | "axys://newtab" => Some(build_newtab(on_search, on_about).upcast()),

        "axys://welcome" => Some(build_welcome().upcast()),

        "axys://privacy" => Some(build_privacy().upcast()),

        "axys://about" => Some(build_about().upcast()),

        "axys://settings" => Some(build_settings(state, on_extensions_changed).upcast()),

        "axys://tools" => Some(build_tools().upcast()),

        _ => None,
    }
}
