use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder, menu::{Menu, MenuEvent, MenuItem}
};

pub fn icon() -> TrayIcon {

    let tray_menu = Menu::new();
    let show_item = MenuItem::with_id("Show", "Show", true, None);
    let quit_item = MenuItem::with_id("Quit", "Quit", true, None);

    tray_menu.append(&show_item).unwrap();
    tray_menu.append(&quit_item).unwrap();

    let icon_bytes = include_bytes!("mash.png");
    let icon = load_icon(icon_bytes);  // Call load_icon with the bytes

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("SpotifyBinds")
        .with_icon(icon)
        .build()
        .unwrap();
    
    _tray_icon
}

pub fn load_icon(bytes: &[u8]) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(bytes)
            .expect("Failed to open icon")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to create icon")
}