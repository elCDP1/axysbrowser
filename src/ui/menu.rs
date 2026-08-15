use gtk::{MenuButton, gio};

pub fn build_menu() -> MenuButton {
    let navigation = gio::Menu::new();

    navigation.append(Some("New tab"), Some("win.new-tab"));
    navigation.append(Some("New window"), Some("win.new-window"));

    let menu = gio::Menu::new();

    menu.append_submenu(Some("Navigation"), &navigation);
    menu.append(Some("Downloads"), Some("win.downloads"));
    menu.append(Some("Privacy"), Some("win.privacy"));
    menu.append(Some("Tools"), Some("win.tools"));
    menu.append(Some("Settings"), Some("win.settings"));

    MenuButton::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text("Menu")
        .menu_model(&menu)
        .build()
}
