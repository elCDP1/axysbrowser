use gtk::prelude::*;
use gtk::{Box, Button, Entry, Label, Orientation, Separator};
use std::rc::Rc;

pub fn build_newtab(on_search: Rc<dyn Fn(String)>, on_about: Rc<dyn Fn()>) -> Box {
    let page = Box::new(Orientation::Vertical, 0);

    page.set_vexpand(true);
    page.set_hexpand(true);

    let content = Box::new(Orientation::Vertical, 0);

    content.set_halign(gtk::Align::Center);

    content.set_valign(gtk::Align::Start);

    content.set_vexpand(true);
    content.set_hexpand(true);

    content.set_margin_top(110);

    let logo = Button::with_label("axys");

    logo.add_css_class("newtab-logo");
    logo.add_css_class("flat");

    logo.set_tooltip_text(Some("About axysBrowser"));

    logo.set_halign(gtk::Align::Center);

    logo.set_margin_bottom(48);

    {
        let on_about = on_about.clone();

        logo.connect_clicked(move |_| {
            on_about();
        });
    }

    let slogan = Label::new(Some("What are you looking for?"));

    slogan.add_css_class("title-2");

    slogan.set_halign(gtk::Align::Center);

    slogan.set_margin_bottom(36);

    let search = Entry::builder()
        .placeholder_text("Search or enter URL")
        .width_chars(42)
        .max_width_chars(54)
        .halign(gtk::Align::Center)
        .build();

    gtk::prelude::EntryExt::set_alignment(&search, 0.5);

    search.add_css_class("search-main");

    {
        let callback = on_search.clone();

        search.connect_activate(move |entry| {
            callback(entry.text().to_string());
        });
    }

    content.append(&logo);
    content.append(&slogan);
    content.append(&search);

    let footer = Box::new(Orientation::Vertical, 6);

    footer.set_halign(gtk::Align::Center);

    footer.set_margin_top(12);
    footer.set_margin_bottom(12);

    let separator = Separator::new(Orientation::Horizontal);

    separator.set_opacity(0.25);

    let copyright = Label::new(Some("© 2026 axysBrowser contributors · beta-1.0"));

    copyright.add_css_class("dim-label");

    let links = Label::new(Some("About · Privacy · Settings"));

    links.add_css_class("dim-label");

    footer.append(&separator);
    footer.append(&copyright);
    footer.append(&links);

    page.append(&content);
    page.append(&footer);

    page
}
