use gtk::prelude::*;
use gtk::{
    Box, Button, CenterBox, Entry, EventControllerFocus, Image, Orientation, Spinner, Stack,
};
use std::cell::RefCell;
use std::rc::Rc;
use webkit6::{WebView, prelude::WebViewExt};

use super::menu::build_menu;
use crate::app_state::AppState;
use crate::browser::engine::BrowserEngine;

pub struct Toolbar {
    pub root: CenterBox,
    pub address: Entry,
    reload_stack: Stack,
    spinner: Spinner,
    back: Button,
    forward: Button,
    extensions: Button,
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
            .tooltip_text("Back")
            .build();

        let forward = Button::builder()
            .icon_name("go-next-symbolic")
            .tooltip_text("Forward")
            .build();

        let reload_stack = Stack::new();

        reload_stack.set_transition_type(gtk::StackTransitionType::None);

        let reload_image = Image::from_icon_name("view-refresh-symbolic");

        reload_image.set_pixel_size(16);

        let reload_button = Button::builder().tooltip_text("Reload").build();

        let spinner = Spinner::new();

        spinner.set_size_request(16, 16);

        reload_stack.add_named(&reload_image, Some("reload"));

        reload_stack.add_named(&spinner, Some("loading"));

        reload_stack.set_visible_child_name("reload");

        reload_button.set_child(Some(&reload_stack));

        back.add_css_class("flat");
        forward.add_css_class("flat");
        reload_button.add_css_class("flat");

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

            reload_button.connect_clicked(move |_| {
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
        navigation.append(&reload_button);

        let address = Entry::builder()
            .placeholder_text("Enter URL")
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

        let extensions = Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Extensions")
            .build();

        extensions.add_css_class("flat");

        extensions.set_visible(state.settings.borrow().show_extensions);

        let menu = build_menu();

        menu.add_css_class("flat");

        right.append(&extensions);
        right.append(&menu);

        toolbar.set_start_widget(Some(&navigation));

        toolbar.set_center_widget(Some(&address));

        toolbar.set_end_widget(Some(&right));

        Self {
            root: toolbar,
            address,
            reload_stack,
            spinner,
            back,
            forward,
            extensions,
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
}
