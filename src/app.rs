use gtk::Application;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app_state::AppState;
use crate::ui::window::build_window;

pub fn run() {
    // Known WebKitGTK6 + Mesa DMA-BUF renderer bug: on some Linux setups
    // (notably Mesa/Intel/AMD combos also affecting XFCE/X11 sessions), the
    // browser freezes entirely — not just the tab — on pages that re-render
    // very frequently while interacting with them, like a chat app's text
    // editor. Disabling the DMA-BUF renderer trades a bit of GPU-accelerated
    // compositing for stability, and is the standard fix distros/WebKit
    // itself recommend for this. Must be set before WebKit initializes,
    // so this has to happen before the GTK Application is even built.
    unsafe {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let app = Application::builder()
        .application_id("com.axys.axysbrowser")
        .build();

    let state_slot = Rc::new(RefCell::new(None::<Rc<AppState>>));

    {
        let state_slot = state_slot.clone();

        app.connect_startup(move |app| {
            let state = AppState::load(app);

            *state_slot.borrow_mut() = Some(state);
        });
    }

    {
        let state_slot = state_slot.clone();

        app.connect_activate(move |app| {
            let Some(state) = state_slot.borrow().clone() else {
                return;
            };

            build_window(app, state);
        });
    }

    app.run();
}
