use gtk::prelude::*;
use gtk::{
    Align, Box, Button, CenterBox, Entry, EventControllerFocus, Image, MenuButton, Orientation,
    Popover, Spinner, Stack,
};
use std::cell::RefCell;
use std::rc::Rc;
use webkit6::{WebView, prelude::WebViewExt};

use super::menu::{build_menu, build_menu_model};
use crate::app_state::AppState;
use crate::browser::downloads::DownloadStatus;
use crate::browser::engine::BrowserEngine;
use crate::internal::pages::downloads::build_row;

pub struct Toolbar {
    pub root: CenterBox,
    pub address: Entry,
    reload_stack: Stack,
    spinner: Spinner,
    back: Button,
    forward: Button,
    reload: Button,
    extensions: Button,
    menu: MenuButton,
    downloads: MenuButton,
    _downloads_refresh: Rc<dyn Fn()>,
}

impl Toolbar {
    pub fn new(
        current_web_view: Rc<RefCell<Option<WebView>>>,
        on_navigate: Rc<dyn Fn(String)>,
        state: Rc<AppState>,
    ) -> Self {
        let toolbar = CenterBox::new();

        toolbar.set_margin_top(3);
        toolbar.set_margin_bottom(7);
        toolbar.set_margin_start(8);
        toolbar.set_margin_end(8);

        let navigation = Box::new(Orientation::Horizontal, 2);

        let back = Button::builder()
            .icon_name("go-previous-symbolic")
            .tooltip_text(rust_i18n::t!("app.back").as_ref())
            .build();

        let forward = Button::builder()
            .icon_name("go-next-symbolic")
            .tooltip_text(rust_i18n::t!("app.forward").as_ref())
            .build();

        let reload_stack = Stack::new();

        reload_stack.set_transition_type(gtk::StackTransitionType::None);

        let reload_image = Image::from_icon_name("view-refresh-symbolic");

        reload_image.set_pixel_size(16);

        let reload = Button::builder()
            .tooltip_text(rust_i18n::t!("app.reload").as_ref())
            .build();

        let spinner = Spinner::new();

        spinner.set_size_request(16, 16);

        reload_stack.add_named(&reload_image, Some("reload"));

        reload_stack.add_named(&spinner, Some("loading"));

        reload_stack.set_visible_child_name("reload");

        reload.set_child(Some(&reload_stack));

        back.add_css_class("flat");
        forward.add_css_class("flat");
        reload.add_css_class("flat");

        {
            let current = current_web_view.clone();

            back.connect_clicked(move |_| {
                if let Some(view) = current.borrow().as_ref() {
                    BrowserEngine::back(view);
                }
            });
        }

        {
            let current = current_web_view.clone();

            forward.connect_clicked(move |_| {
                if let Some(view) = current.borrow().as_ref() {
                    BrowserEngine::forward(view);
                }
            });
        }

        {
            let current = current_web_view.clone();

            reload.connect_clicked(move |_| {
                if let Some(view) = current.borrow().as_ref() {
                    if view.is_loading() {
                        view.stop_loading();
                    } else {
                        BrowserEngine::reload(view);
                    }
                }
            });
        }

        navigation.append(&back);
        navigation.append(&forward);
        navigation.append(&reload);

        let address = Entry::builder()
            .placeholder_text(rust_i18n::t!("app.enter_url").as_ref())
            .hexpand(true)
            .build();

        address.add_css_class("address-top");

        {
            let on_navigate = on_navigate.clone();

            address.connect_activate(move |entry| {
                on_navigate(entry.text().to_string());
            });
        }

        let focus_controller = EventControllerFocus::new();

        {
            let address = address.clone();

            focus_controller.connect_enter(move |_| {
                if address.text() == "axys://newtab" {
                    address.set_text("");
                }
            });
        }

        address.add_controller(focus_controller);

        let right = Box::new(Orientation::Horizontal, 2);

        let downloads_popover = Popover::new();

        downloads_popover.set_has_arrow(true);

        let downloads_content = Box::new(Orientation::Vertical, 8);

        downloads_content.set_margin_top(10);

        downloads_content.set_margin_bottom(10);

        downloads_content.set_margin_start(10);

        downloads_content.set_margin_end(10);

        downloads_content.set_size_request(280, -1);

        downloads_popover.set_child(Some(&downloads_content));

        let downloads = MenuButton::builder()
            .icon_name("folder-download-symbolic")
            .tooltip_text(rust_i18n::t!("app.downloads").as_ref())
            .popover(&downloads_popover)
            .build();

        downloads.add_css_class("flat");

        downloads.set_visible(false);

        let extensions = Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text(rust_i18n::t!("app.extensions").as_ref())
            .build();

        extensions.add_css_class("flat");

        extensions.set_visible(state.settings.borrow().show_extensions);

        let menu = build_menu();

        menu.add_css_class("flat");

        right.append(&downloads);
        right.append(&extensions);
        right.append(&menu);

        toolbar.set_start_widget(Some(&navigation));

        toolbar.set_center_widget(Some(&address));

        toolbar.set_end_widget(Some(&right));

        let downloads_refresh: Rc<dyn Fn()> = {
            let downloads_button = downloads.clone();

            let downloads_content = downloads_content.clone();

            let downloads_popover = downloads_popover.clone();

            let manager = state.downloads.clone();

            let on_navigate = on_navigate.clone();

            Rc::new(move || {
                let entries = manager.entries();

                let active = entries
                    .iter()
                    .filter(|entry| entry.status == DownloadStatus::InProgress)
                    .count();

                downloads_button.set_visible(!entries.is_empty());

                let tooltip = if active > 0 {
                    format!("{} ({active} active)", rust_i18n::t!("app.downloads"))
                } else {
                    rust_i18n::t!("app.downloads").to_string()
                };

                downloads_button.set_tooltip_text(Some(&tooltip));

                while let Some(child) = downloads_content.first_child() {
                    downloads_content.remove(&child);
                }

                let title = gtk::Label::new(Some(&rust_i18n::t!("downloads.title")));

                title.add_css_class("title-4");

                title.set_halign(Align::Start);

                downloads_content.append(&title);

                for entry in entries.iter().rev().take(5) {
                    downloads_content.append(&build_row(entry, &manager));
                }

                let see_all = Button::with_label(&rust_i18n::t!("downloads.see_all"));

                see_all.add_css_class("flat");

                see_all.set_halign(Align::Fill);

                {
                    let on_navigate = on_navigate.clone();

                    let downloads_popover = downloads_popover.clone();

                    see_all.connect_clicked(move |_| {
                        downloads_popover.popdown();

                        on_navigate("axys://downloads".to_string());
                    });
                }

                downloads_content.append(&see_all);
            })
        };

        downloads_refresh();

        state.downloads.subscribe(&downloads_refresh);

        Self {
            root: toolbar,
            address,
            reload_stack,
            spinner,
            back,
            forward,
            reload,
            extensions,
            menu,
            downloads,
            _downloads_refresh: downloads_refresh,
        }
    }

    pub fn set_loading(&self, loading: bool) {
        if loading {
            self.reload_stack.set_visible_child_name("loading");

            self.spinner.start();
        } else {
            self.spinner.stop();

            self.reload_stack.set_visible_child_name("reload");
        }
    }

    pub fn set_navigation_state(&self, can_go_back: bool, can_go_forward: bool) {
        self.back.set_sensitive(can_go_back);

        self.forward.set_sensitive(can_go_forward);
    }

    pub fn set_extensions_visible(&self, visible: bool) {
        self.extensions.set_visible(visible);
    }

    pub fn refresh_language(&self) {
        self.back
            .set_tooltip_text(Some(rust_i18n::t!("app.back").as_ref()));

        self.forward
            .set_tooltip_text(Some(rust_i18n::t!("app.forward").as_ref()));

        self.reload
            .set_tooltip_text(Some(rust_i18n::t!("app.reload").as_ref()));

        self.address
            .set_placeholder_text(Some(rust_i18n::t!("app.enter_url").as_ref()));

        self.extensions
            .set_tooltip_text(Some(rust_i18n::t!("app.extensions").as_ref()));

        self.downloads
            .set_tooltip_text(Some(rust_i18n::t!("app.downloads").as_ref()));

        self.menu
            .set_tooltip_text(Some(rust_i18n::t!("app.menu").as_ref()));

        self.menu.set_menu_model(Some(&build_menu_model()));

        (self._downloads_refresh)();
    }
}
