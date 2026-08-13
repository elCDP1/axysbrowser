use gtk::prelude::*;
use gtk::{Box, Button, Label, Orientation};

pub fn build_tools() -> Box {
    let page = Box::new(Orientation::Vertical, 14);

    page.set_margin_top(28);
    page.set_margin_bottom(28);
    page.set_margin_start(28);
    page.set_margin_end(28);

    page.set_vexpand(true);
    page.set_hexpand(true);

    let title = Label::new(Some("Tools"));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Start);

    let inspector = Button::with_label("Web Inspector");
    inspector.set_halign(gtk::Align::Start);

    let downloads = Button::with_label("Downloads");
    downloads.set_halign(gtk::Align::Start);

    let browsing_data = Button::with_label("Browsing data");
    browsing_data.set_halign(gtk::Align::Start);

    let note = Label::new(Some("More browser tools will be added here."));

    note.add_css_class("dim-label");
    note.set_halign(gtk::Align::Start);

    page.append(&title);
    page.append(&inspector);
    page.append(&downloads);
    page.append(&browsing_data);
    page.append(&note);

    page
}
