use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, Button, Label, Orientation, ProgressBar, ScrolledWindow};
use std::rc::Rc;

use crate::app_state::AppState;
use crate::browser::downloads::{DownloadEntry, DownloadManager, DownloadStatus};

/// Builds a single download row (filename, status/progress, and the
/// relevant action button). Shared between the full `axys://downloads`
/// page and the toolbar's quick downloads popover so they stay visually
/// consistent and don't duplicate this logic.
pub fn build_row(entry: &DownloadEntry, manager: &DownloadManager) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.set_hexpand(true);

    let info = GtkBox::new(Orientation::Vertical, 3);
    info.set_hexpand(true);

    let name = Label::new(Some(&entry.filename));
    name.set_halign(Align::Start);
    name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    info.append(&name);

    let status_text = match &entry.status {
        DownloadStatus::InProgress => format!("{:.0}%", entry.progress * 100.0),
        DownloadStatus::Completed => "Completed".to_string(),
        DownloadStatus::Cancelled => "Cancelled".to_string(),
        DownloadStatus::Failed(message) => format!("Failed: {message}"),
    };

    let status = Label::new(Some(&status_text));
    status.add_css_class("dim-label");
    status.set_halign(Align::Start);
    info.append(&status);

    if entry.status == DownloadStatus::InProgress {
        let bar = ProgressBar::new();
        bar.set_fraction(entry.progress);
        bar.set_hexpand(true);
        info.append(&bar);
    }

    row.append(&info);

    match &entry.status {
        DownloadStatus::InProgress => {
            let cancel = Button::with_label("Cancel");
            cancel.add_css_class("flat");

            let manager = manager.clone();
            let id = entry.id;

            cancel.connect_clicked(move |_| {
                manager.cancel(id);
            });

            row.append(&cancel);
        }

        DownloadStatus::Completed => {
            let show = Button::from_icon_name("folder-symbolic");
            show.add_css_class("flat");
            show.set_tooltip_text(Some("Show in folder"));

            let path = entry.path.clone();

            show.connect_clicked(move |_| {
                DownloadManager::open_containing_folder(&path);
            });

            row.append(&show);
        }

        _ => {}
    }

    row
}

pub fn build_downloads(state: Rc<AppState>) -> GtkBox {
    let page = GtkBox::new(Orientation::Vertical, 14);

    page.set_margin_top(28);
    page.set_margin_bottom(28);
    page.set_margin_start(28);
    page.set_margin_end(28);

    page.set_vexpand(true);
    page.set_hexpand(true);

    let title = Label::new(Some("Downloads"));
    title.add_css_class("title-1");
    title.set_halign(Align::Start);

    let list = GtkBox::new(Orientation::Vertical, 10);

    let scroller = ScrolledWindow::new();
    scroller.set_child(Some(&list));
    scroller.set_vexpand(true);
    scroller.set_hexpand(true);

    page.append(&title);
    page.append(&scroller);

    let refresh: Rc<dyn Fn()> = {
        let list = list.clone();
        let manager = state.downloads.clone();

        Rc::new(move || {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }

            let entries = manager.entries();

            if entries.is_empty() {
                let empty = Label::new(Some("No downloads yet."));
                empty.add_css_class("dim-label");
                empty.set_halign(Align::Start);
                list.append(&empty);
                return;
            }

            for entry in entries.iter().rev() {
                list.append(&build_row(entry, &manager));
            }
        })
    };

    refresh();

    state.downloads.subscribe(&refresh);

    // Keeps `refresh` (and everything it captured) alive for as long as this
    // page widget exists, so the weak subscription in DownloadManager stays valid.
    // SAFETY: we only ever set this key once per widget and never read it back
    // with `get_data`/`steal_data`, so there's no risk of a type mismatch on
    // retrieval (which is what makes `set_data` unsafe in this glib version).
    unsafe {
        page.set_data("downloads-refresh-keepalive", refresh);
    }

    page
}
