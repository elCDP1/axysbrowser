use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, CssProvider, EventControllerKey, Orientation, Stack,
    StackTransitionType, gdk, gio, glib::Propagation, style_context_add_provider_for_display,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use webkit6::{LoadEvent, NetworkSession, WebView, prelude::WebViewExt};

use crate::app_state::AppState;
use crate::browser::address::{SearchConfig, resolve_input};
use crate::browser::engine::BrowserEngine;
use crate::browser::tabs::Tab;
use crate::internal::router;

use super::tabs::TabBar;
use super::toolbar::Toolbar;

fn maximize_window(window: &ApplicationWindow) {
    window.maximize();
}

pub fn build_window(app: &Application, state: Rc<AppState>) {
    let session = state.network_session.clone();

    let search = state.search.clone();

    build_browser_window(app, state, session, search, false);
}

pub fn build_private_window(app: &Application, state: Rc<AppState>) {
    let session = NetworkSession::new_ephemeral();

    session.set_itp_enabled(true);

    session.set_persistent_credential_storage_enabled(false);

    let mut private_search = state.search.borrow().clone();

    if private_search.engines.contains_key("brave") {
        private_search.default_engine = "brave".to_string();
    }

    let search = Rc::new(RefCell::new(private_search));

    build_browser_window(app, state, session, search, true);
}

fn build_browser_window(
    app: &Application,
    state: Rc<AppState>,
    session: NetworkSession,
    search: Rc<RefCell<SearchConfig>>,
    private_mode: bool,
) {
    let window_title = if private_mode {
        "Privacy mode"
    } else {
        "axysBrowser"
    };

    let window = ApplicationWindow::builder()
        .application(app)
        .title(window_title)
        .resizable(true)
        .build();

    let css = CssProvider::new();

    css.load_from_data(
        r#"
        .address-top {
            min-height: 18px;
            padding: 5px 12px;
            border-radius: 16px;
            border: 1px solid alpha(@theme_fg_color, 0.08);
            background: alpha(@theme_fg_color, 0.045);
            box-shadow: none;
            font-size: 0.90em;
        }

        .address-top:focus {
            border-color: alpha(@theme_selected_bg_color, 0.45);
            background: alpha(@theme_fg_color, 0.07);
        }

        .search-main {
            min-height: 34px;
            padding: 10px 16px;
            border-radius: 18px;
            border: 1px solid alpha(@theme_fg_color, 0.10);
            background: alpha(@theme_fg_color, 0.055);
            box-shadow: none;
            font-size: 0.92em;
        }

        .search-main:focus {
            border-color: alpha(@theme_selected_bg_color, 0.55);
            background: alpha(@theme_fg_color, 0.08);
        }

        .tab {
            min-height: 30px;
            padding: 2px 4px 2px 9px;
            border-radius: 9px;
            background: transparent;
        }

        .tab:hover {
            background: alpha(@theme_fg_color, 0.06);
        }

        .tab.active {
            background: alpha(@theme_fg_color, 0.10);
        }

        .tab-select {
            padding: 2px 5px;
            border-radius: 7px;
            background: transparent;
            box-shadow: none;
        }

        .tab-close {
            opacity: 0;
            min-width: 20px;
            min-height: 20px;
            padding: 0;
            border-radius: 6px;
        }

        .tab:hover .tab-close,
        .tab.active .tab-close {
            opacity: 1;
        }

        .newtab-logo {
            font-size: 3.8em;
            font-weight: 600;
            padding: 4px 18px;
            border-radius: 12px;
            background: transparent;
            box-shadow: none;
        }

        .newtab-logo:hover {
            background: alpha(@theme_fg_color, 0.06);
        }

        .newtab-logo:active {
            background: alpha(@theme_fg_color, 0.10);
        }

        button.flat {
            min-width: 30px;
            min-height: 30px;
            padding: 4px;
            border-radius: 9px;
            box-shadow: none;
        }

        button.flat:hover {
            background: alpha(@theme_fg_color, 0.07);
        }

        button.flat:active {
            background: alpha(@theme_fg_color, 0.12);
        }

        .private-banner {
            padding: 6px 12px;
            border-radius: 12px;
            background: alpha(@theme_selected_bg_color, 0.10);
        }
        "#,
    );

    if let Some(display) = gdk::Display::default() {
        style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let root = Box::new(Orientation::Vertical, 0);

    root.set_hexpand(true);
    root.set_vexpand(true);

    if private_mode {
        let banner = gtk::Label::new(Some("Privacy mode · temporary website data · Brave Search"));

        banner.add_css_class("private-banner");

        banner.set_margin_top(3);
        banner.set_margin_bottom(2);
        banner.set_margin_start(8);
        banner.set_margin_end(8);

        root.append(&banner);
    }

    let tabs = Rc::new(RefCell::new(Vec::<Tab>::new()));

    let active_id = Rc::new(Cell::new(usize::MAX));

    let next_id = Rc::new(Cell::new(0usize));

    let top_stack = Stack::new();

    top_stack.set_hexpand(true);
    top_stack.set_vexpand(true);
    top_stack.set_halign(gtk::Align::Fill);
    top_stack.set_valign(gtk::Align::Fill);
    top_stack.set_transition_type(StackTransitionType::None);

    let current_web_view = Rc::new(RefCell::new(None::<WebView>));

    let tabbar_slot: Rc<RefCell<Option<TabBar>>> = Rc::new(RefCell::new(None));

    let toolbar_slot: Rc<RefCell<Option<Toolbar>>> = Rc::new(RefCell::new(None));

    let select_tab: Rc<dyn Fn(usize)> = {
        let tabs = tabs.clone();

        let active_id = active_id.clone();

        let top_stack = top_stack.clone();

        let current_web_view = current_web_view.clone();

        let toolbar_slot = toolbar_slot.clone();

        let tabbar_slot = tabbar_slot.clone();

        Rc::new(move |id| {
            let tab = {
                let tabs = tabs.borrow();

                tabs.iter().find(|tab| tab.id == id).map(|tab| {
                    (
                        tab.content.clone(),
                        tab.web_view.clone(),
                        tab.uri.clone(),
                        tab.can_go_back(),
                        tab.can_go_forward(),
                    )
                })
            };

            let Some((content, web_view, uri, can_go_back, can_go_forward)) = tab else {
                return;
            };

            active_id.set(id);

            top_stack.set_visible_child(&content);

            *current_web_view.borrow_mut() = Some(web_view.clone());

            if let Some(toolbar) = toolbar_slot.borrow().as_ref() {
                toolbar.address.set_text(&uri);

                toolbar.set_loading(web_view.is_loading());

                toolbar.set_navigation_state(
                    can_go_back || web_view.can_go_back(),
                    can_go_forward || web_view.can_go_forward(),
                );
            }

            if let Some(tabbar) = tabbar_slot.borrow().as_ref() {
                tabbar.refresh(&tabs.borrow(), id);
            }
        })
    };

    let close_tab: Rc<dyn Fn(usize)> = {
        let tabs = tabs.clone();

        let active_id = active_id.clone();

        let top_stack = top_stack.clone();

        let toolbar_slot = toolbar_slot.clone();

        let tabbar_slot = tabbar_slot.clone();

        let current_web_view = current_web_view.clone();

        let select_tab = select_tab.clone();

        Rc::new(move |id| {
            if tabs.borrow().len() == 1 {
                let (content, web_view) = {
                    let mut tabs_ref = tabs.borrow_mut();

                    let Some(tab) = tabs_ref.iter_mut().find(|tab| tab.id == id) else {
                        return;
                    };

                    tab.title = "New Tab".to_string();

                    tab.uri = "axys://newtab".to_string();

                    tab.history = vec!["axys://newtab".to_string()];

                    tab.history_index = 0;

                    tab.content.set_visible_child_name("newtab");

                    (tab.content.clone(), tab.web_view.clone())
                };

                active_id.set(id);

                top_stack.set_visible_child(&content);

                *current_web_view.borrow_mut() = Some(web_view);

                if let Some(toolbar) = toolbar_slot.borrow().as_ref() {
                    toolbar.address.set_text("axys://newtab");

                    toolbar.set_loading(false);

                    toolbar.set_navigation_state(false, false);
                }

                if let Some(tabbar) = tabbar_slot.borrow().as_ref() {
                    tabbar.refresh(&tabs.borrow(), id);
                }

                return;
            }

            let was_active = active_id.get() == id;

            let removed = {
                let mut tabs_ref = tabs.borrow_mut();

                tabs_ref
                    .iter()
                    .position(|tab| tab.id == id)
                    .map(|index| tabs_ref.remove(index))
            };

            let Some(removed) = removed else {
                return;
            };

            top_stack.remove(&removed.content);

            if was_active {
                let new_id = tabs.borrow().last().map(|tab| tab.id);

                if let Some(new_id) = new_id {
                    select_tab(new_id);
                }
            }

            if let Some(tabbar) = tabbar_slot.borrow().as_ref() {
                tabbar.refresh(&tabs.borrow(), active_id.get());
            }
        })
    };

    let navigate_tab: Rc<dyn Fn(usize, String)> = {
        let tabs = tabs.clone();

        let top_stack = top_stack.clone();

        let active_id = active_id.clone();

        let toolbar_slot = toolbar_slot.clone();

        let search = search.clone();

        Rc::new(move |id, input| {
            let target = {
                let config = search.borrow();

                resolve_input(&input, &config)
            };

            if let Some(page) = router::page_name(&target) {
                let (content, can_go_back, can_go_forward) = {
                    let mut tabs_ref = tabs.borrow_mut();

                    let Some(tab) = tabs_ref.iter_mut().find(|tab| tab.id == id) else {
                        return;
                    };

                    tab.content.set_visible_child_name(page);

                    tab.uri = target.clone();

                    tab.push_history(target.clone());

                    (tab.content.clone(), tab.can_go_back(), tab.can_go_forward())
                };

                if active_id.get() == id {
                    top_stack.set_visible_child(&content);

                    if let Some(toolbar) = toolbar_slot.borrow().as_ref() {
                        toolbar.address.set_text(&target);

                        toolbar.set_loading(false);

                        toolbar.set_navigation_state(can_go_back, can_go_forward);
                    }
                }

                return;
            }

            if !target.starts_with("http://") && !target.starts_with("https://") {
                return;
            }

            let (content, web_view, can_go_back, can_go_forward) = {
                let mut tabs_ref = tabs.borrow_mut();

                let Some(tab) = tabs_ref.iter_mut().find(|tab| tab.id == id) else {
                    return;
                };

                if tab.uri != target {
                    tab.push_history(target.clone());
                }

                tab.uri = target.clone();

                tab.content.set_visible_child_name("web");

                (
                    tab.content.clone(),
                    tab.web_view.clone(),
                    tab.can_go_back(),
                    tab.can_go_forward(),
                )
            };

            if active_id.get() == id {
                top_stack.set_visible_child(&content);

                if let Some(toolbar) = toolbar_slot.borrow().as_ref() {
                    toolbar.address.set_text(&target);

                    toolbar.set_loading(true);

                    toolbar.set_navigation_state(
                        can_go_back,
                        can_go_forward || web_view.can_go_forward(),
                    );
                }
            }

            BrowserEngine::load(&web_view, &target);
        })
    };

    let go_back: Rc<dyn Fn()> = {
        let tabs = tabs.clone();

        let active_id = active_id.clone();

        let top_stack = top_stack.clone();

        let toolbar_slot = toolbar_slot.clone();

        Rc::new(move || {
            let id = active_id.get();

            let target = {
                let mut tabs_ref = tabs.borrow_mut();

                let Some(tab) = tabs_ref.iter_mut().find(|tab| tab.id == id) else {
                    return;
                };

                tab.go_back()
            };

            let Some(target) = target else {
                return;
            };

            let tab_data = {
                let tabs_ref = tabs.borrow();

                tabs_ref.iter().find(|tab| tab.id == id).map(|tab| {
                    (
                        tab.content.clone(),
                        tab.web_view.clone(),
                        tab.can_go_back(),
                        tab.can_go_forward(),
                    )
                })
            };

            let Some((content, web_view, can_go_back, can_go_forward)) = tab_data else {
                return;
            };

            if let Some(page) = router::page_name(&target) {
                content.set_visible_child_name(page);

                if let Some(toolbar) = toolbar_slot.borrow().as_ref() {
                    toolbar.address.set_text(&target);

                    toolbar.set_loading(false);

                    toolbar.set_navigation_state(can_go_back, can_go_forward);
                }

                top_stack.set_visible_child(&content);
            } else {
                BrowserEngine::load(&web_view, &target);
            }
        })
    };

    let go_forward: Rc<dyn Fn()> = {
        let tabs = tabs.clone();

        let active_id = active_id.clone();

        let top_stack = top_stack.clone();

        let toolbar_slot = toolbar_slot.clone();

        Rc::new(move || {
            let id = active_id.get();

            let target = {
                let mut tabs_ref = tabs.borrow_mut();

                let Some(tab) = tabs_ref.iter_mut().find(|tab| tab.id == id) else {
                    return;
                };

                tab.go_forward()
            };

            let Some(target) = target else {
                return;
            };

            let tab_data = {
                let tabs_ref = tabs.borrow();

                tabs_ref.iter().find(|tab| tab.id == id).map(|tab| {
                    (
                        tab.content.clone(),
                        tab.web_view.clone(),
                        tab.can_go_back(),
                        tab.can_go_forward(),
                    )
                })
            };

            let Some((content, web_view, can_go_back, can_go_forward)) = tab_data else {
                return;
            };

            if let Some(page) = router::page_name(&target) {
                content.set_visible_child_name(page);

                if let Some(toolbar) = toolbar_slot.borrow().as_ref() {
                    toolbar.address.set_text(&target);

                    toolbar.set_loading(false);

                    toolbar.set_navigation_state(can_go_back, can_go_forward);
                }

                top_stack.set_visible_child(&content);
            } else {
                BrowserEngine::load(&web_view, &target);
            }
        })
    };

    let new_tab: Rc<dyn Fn() -> usize> = {
        let tabs = tabs.clone();

        let next_id = next_id.clone();

        let active_id = active_id.clone();

        let top_stack = top_stack.clone();

        let current_web_view = current_web_view.clone();

        let tabbar_slot = Rc::downgrade(&tabbar_slot);

        let toolbar_slot = Rc::downgrade(&toolbar_slot);

        let navigate_tab = navigate_tab.clone();

        let state = state.clone();

        let session = session.clone();

        Rc::new(move || {
            let id = next_id.get();

            next_id.set(id + 1);

            let content = Stack::new();

            content.set_hexpand(true);
            content.set_vexpand(true);
            content.set_halign(gtk::Align::Fill);
            content.set_valign(gtk::Align::Fill);
            content.set_transition_type(StackTransitionType::None);

            let web_view = WebView::builder().network_session(&session).build();

            web_view.set_hexpand(true);
            web_view.set_vexpand(true);
            web_view.set_halign(gtk::Align::Fill);
            web_view.set_valign(gtk::Align::Fill);

            BrowserEngine::configure(&web_view);

            if private_mode
                && let Some(settings) = webkit6::prelude::WebViewExt::settings(&web_view)
            {
                settings.set_enable_developer_extras(false);
            }

            content.add_named(&web_view, Some("web"));

            let local_search = {
                let navigate_tab = navigate_tab.clone();

                Rc::new(move |input: String| {
                    navigate_tab(id, input);
                })
            };

            let local_about = {
                let navigate_tab = navigate_tab.clone();

                Rc::new(move || {
                    navigate_tab(id, "axys://about".to_string());
                })
            };

            let local_extensions_changed = {
                let toolbar_slot = toolbar_slot.clone();

                Rc::new(move |visible: bool| {
                    if let Some(toolbar_slot) = toolbar_slot.upgrade()
                        && let Some(toolbar) = toolbar_slot.borrow().as_ref()
                    {
                        toolbar.set_extensions_visible(visible);
                    }
                })
            };

            for uri in [
                "axys://welcome",
                "axys://newtab",
                "axys://privacy",
                "axys://about",
                "axys://settings",
                "axys://tools",
            ] {
                if let Some(widget) = router::route(
                    uri,
                    local_search.clone(),
                    local_about.clone(),
                    local_extensions_changed.clone(),
                    state.clone(),
                ) && let Some(name) = router::page_name(uri)
                {
                    content.add_named(&widget, Some(name));
                }
            }

            let first_page = if private_mode { "privacy" } else { "newtab" };

            content.set_visible_child_name(first_page);

            let tab_name = format!("tab-{id}");

            top_stack.add_named(&content, Some(&tab_name));

            let tab = Tab::new(id, content.clone(), web_view.clone());

            tabs.borrow_mut().push(tab);

            active_id.set(id);

            top_stack.set_visible_child(&content);

            *current_web_view.borrow_mut() = Some(web_view.clone());

            if let Some(toolbar_slot) = toolbar_slot.upgrade()
                && let Some(toolbar) = toolbar_slot.borrow().as_ref()
            {
                toolbar.address.set_text("axys://newtab");

                toolbar.set_loading(false);

                toolbar.set_navigation_state(false, false);
            }

            {
                let tabs = tabs.clone();

                let active_id = active_id.clone();

                let toolbar_slot = toolbar_slot.clone();

                let tabbar_slot = tabbar_slot.clone();

                web_view.connect_uri_notify(move |view| {
                    let Some(uri) = view.uri().map(|uri| uri.to_string()) else {
                        return;
                    };

                    if let Some(tab) = tabs.borrow_mut().iter_mut().find(|tab| tab.id == id) {
                        tab.uri = uri.clone();
                    }

                    if active_id.get() == id
                        && let Some(toolbar_slot) = toolbar_slot.upgrade()
                        && let Some(toolbar) = toolbar_slot.borrow().as_ref()
                    {
                        toolbar.address.set_text(&uri);
                    }

                    if let Some(tabbar_slot) = tabbar_slot.upgrade()
                        && let Some(tabbar) = tabbar_slot.borrow().as_ref()
                    {
                        tabbar.refresh(&tabs.borrow(), active_id.get());
                    }
                });
            }

            {
                let tabs = tabs.clone();

                let active_id = active_id.clone();

                let tabbar_slot = tabbar_slot.clone();

                web_view.connect_title_notify(move |view| {
                    let title = view
                        .title()
                        .map(|title| title.to_string())
                        .filter(|title| !title.trim().is_empty())
                        .unwrap_or_else(|| {
                            if private_mode {
                                "Privacy".to_string()
                            } else {
                                "New Tab".to_string()
                            }
                        });

                    if let Some(tab) = tabs.borrow_mut().iter_mut().find(|tab| tab.id == id) {
                        tab.title = title;
                    }

                    if let Some(tabbar_slot) = tabbar_slot.upgrade()
                        && let Some(tabbar) = tabbar_slot.borrow().as_ref()
                    {
                        tabbar.refresh(&tabs.borrow(), active_id.get());
                    }
                });
            }

            {
                let active_id = active_id.clone();

                let toolbar_slot = toolbar_slot.clone();

                web_view.connect_is_loading_notify(move |view| {
                    if active_id.get() != id {
                        return;
                    }

                    if let Some(toolbar_slot) = toolbar_slot.upgrade()
                        && let Some(toolbar) = toolbar_slot.borrow().as_ref()
                    {
                        toolbar.set_loading(view.is_loading());
                    }
                });
            }

            {
                let active_id = active_id.clone();

                let toolbar_slot = toolbar_slot.clone();

                web_view.connect_load_changed(move |view, event| {
                    if active_id.get() != id {
                        return;
                    }

                    if let Some(toolbar_slot) = toolbar_slot.upgrade()
                        && let Some(toolbar) = toolbar_slot.borrow().as_ref()
                    {
                        toolbar.set_loading(view.is_loading());

                        match event {
                            LoadEvent::Started
                            | LoadEvent::Committed
                            | LoadEvent::Finished
                            | LoadEvent::Redirected => {
                                toolbar.set_navigation_state(
                                    view.can_go_back(),
                                    view.can_go_forward(),
                                );
                            }

                            _ => {}
                        }
                    }
                });
            }

            if let Some(tabbar_slot) = tabbar_slot.upgrade()
                && let Some(tabbar) = tabbar_slot.borrow().as_ref()
            {
                tabbar.refresh(&tabs.borrow(), id);
            }

            id
        })
    };

    let tabbar = TabBar::new(
        {
            let new_tab = new_tab.clone();

            Rc::new(move || {
                new_tab();
            })
        },
        select_tab.clone(),
        close_tab.clone(),
        {
            let navigate_tab = navigate_tab.clone();

            let active_id = active_id.clone();

            Rc::new(move || {
                navigate_tab(active_id.get(), "axys://about".to_string());
            })
        },
    );

    *tabbar_slot.borrow_mut() = Some(tabbar);

    let toolbar = Toolbar::new(
        current_web_view.clone(),
        {
            let navigate_tab = navigate_tab.clone();

            let active_id = active_id.clone();

            Rc::new(move |input: String| {
                navigate_tab(active_id.get(), input);
            })
        },
        state.clone(),
    );

    *toolbar_slot.borrow_mut() = Some(toolbar);

    if let Some(tabbar) = tabbar_slot.borrow().as_ref() {
        root.append(&tabbar.root);
    }

    if let Some(toolbar) = toolbar_slot.borrow().as_ref() {
        root.append(&toolbar.root);
    }

    root.append(&top_stack);

    window.set_child(Some(&root));

    let keyboard = EventControllerKey::new();

    {
        let new_tab = new_tab.clone();

        let close_tab = close_tab.clone();

        let select_tab = select_tab.clone();

        let tabs = tabs.clone();

        let active_id = active_id.clone();

        let toolbar_slot = toolbar_slot.clone();

        let current_web_view = current_web_view.clone();

        let go_back = go_back.clone();

        let go_forward = go_forward.clone();

        let app_for_shortcuts = app.clone();

        let state_for_shortcuts = state.clone();

        keyboard.connect_key_pressed(move |_, key, _, modifiers| {
            let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);

            let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);

            let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);

            if ctrl {
                if let Some(ch) = key.to_unicode()
                    && matches!(ch.to_ascii_lowercase(), 'n' | 't' | 'w' | 'l' | 'r' | 'p')
                {
                    match ch.to_ascii_lowercase() {
                        'n' => {
                            if shift {
                                build_private_window(
                                    &app_for_shortcuts,
                                    state_for_shortcuts.clone(),
                                );
                            } else {
                                build_window(&app_for_shortcuts, state_for_shortcuts.clone());
                            }

                            return Propagation::Stop;
                        }

                        't' => {
                            new_tab();

                            return Propagation::Stop;
                        }

                        'w' => {
                            close_tab(active_id.get());

                            return Propagation::Stop;
                        }

                        'l' => {
                            if let Some(toolbar) = toolbar_slot.borrow().as_ref() {
                                toolbar.address.grab_focus();

                                toolbar.address.select_region(0, -1);
                            }

                            return Propagation::Stop;
                        }

                        'r' => {
                            if let Some(view) = current_web_view.borrow().as_ref() {
                                if shift {
                                    view.reload_bypass_cache();
                                } else {
                                    BrowserEngine::reload(view);
                                }
                            }

                            return Propagation::Stop;
                        }

                        'p' if shift => {
                            build_private_window(&app_for_shortcuts, state_for_shortcuts.clone());

                            return Propagation::Stop;
                        }

                        _ => {}
                    }
                }

                if key == gdk::Key::Tab {
                    let tabs_ref = tabs.borrow();

                    if tabs_ref.is_empty() {
                        return Propagation::Stop;
                    }

                    let current = tabs_ref
                        .iter()
                        .position(|tab| tab.id == active_id.get())
                        .unwrap_or(0);

                    let next = if shift {
                        if current == 0 {
                            tabs_ref.len() - 1
                        } else {
                            current - 1
                        }
                    } else {
                        (current + 1) % tabs_ref.len()
                    };

                    let next_id = tabs_ref[next].id;

                    drop(tabs_ref);

                    select_tab(next_id);

                    return Propagation::Stop;
                }

                if key == gdk::Key::Page_Up {
                    let tabs_ref = tabs.borrow();

                    if tabs_ref.is_empty() {
                        return Propagation::Stop;
                    }

                    let current = tabs_ref
                        .iter()
                        .position(|tab| tab.id == active_id.get())
                        .unwrap_or(0);

                    let next = if current == 0 {
                        tabs_ref.len() - 1
                    } else {
                        current - 1
                    };

                    let next_id = tabs_ref[next].id;

                    drop(tabs_ref);

                    select_tab(next_id);

                    return Propagation::Stop;
                }

                if key == gdk::Key::Page_Down {
                    let tabs_ref = tabs.borrow();

                    if tabs_ref.is_empty() {
                        return Propagation::Stop;
                    }

                    let current = tabs_ref
                        .iter()
                        .position(|tab| tab.id == active_id.get())
                        .unwrap_or(0);

                    let next = (current + 1) % tabs_ref.len();

                    let next_id = tabs_ref[next].id;

                    drop(tabs_ref);

                    select_tab(next_id);

                    return Propagation::Stop;
                }

                if let Some(ch) = key.to_unicode()
                    && let Some(digit) = ch.to_digit(10)
                    && (1..=9).contains(&digit)
                {
                    let index = digit as usize - 1;

                    let tabs_ref = tabs.borrow();

                    if index < tabs_ref.len() {
                        let id = tabs_ref[index].id;

                        drop(tabs_ref);

                        select_tab(id);
                    }

                    return Propagation::Stop;
                }
            }

            if alt {
                match key {
                    gdk::Key::Left => {
                        go_back();

                        return Propagation::Stop;
                    }

                    gdk::Key::Right => {
                        go_forward();

                        return Propagation::Stop;
                    }

                    _ => {}
                }
            }

            if key == gdk::Key::F5 {
                if let Some(view) = current_web_view.borrow().as_ref() {
                    BrowserEngine::reload(view);
                }

                return Propagation::Stop;
            }

            if key == gdk::Key::Escape
                && let Some(view) = current_web_view.borrow().as_ref()
                && view.is_loading()
            {
                view.stop_loading();

                return Propagation::Stop;
            }

            Propagation::Proceed
        });

        window.add_controller(keyboard);
    }

    {
        let new_tab = new_tab.clone();

        let action = gio::SimpleAction::new("new-tab", None);

        action.connect_activate(move |_, _| {
            new_tab();
        });

        window.add_action(&action);
    }

    {
        let app = app.clone();

        let state = state.clone();

        let action = gio::SimpleAction::new("new-window", None);

        action.connect_activate(move |_, _| {
            build_window(&app, state.clone());
        });

        window.add_action(&action);
    }

    {
        let app = app.clone();

        let state = state.clone();

        let action = gio::SimpleAction::new("privacy", None);

        action.connect_activate(move |_, _| {
            build_private_window(&app, state.clone());
        });

        window.add_action(&action);
    }

    {
        let new_tab = new_tab.clone();

        let navigate_tab = navigate_tab.clone();

        let action = gio::SimpleAction::new("tools", None);

        action.connect_activate(move |_, _| {
            let id = new_tab();

            navigate_tab(id, "axys://tools".to_string());
        });

        window.add_action(&action);
    }

    {
        let new_tab = new_tab.clone();

        let navigate_tab = navigate_tab.clone();

        let action = gio::SimpleAction::new("settings", None);

        action.connect_activate(move |_, _| {
            let id = new_tab();

            navigate_tab(id, "axys://settings".to_string());
        });

        window.add_action(&action);
    }

    let first_tab = new_tab();

    active_id.set(first_tab);

    window.present();

    maximize_window(&window);
}
