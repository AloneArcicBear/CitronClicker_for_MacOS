// #claude
//! menu bar icon (macos). native menu with "show" and "quit"; its actions run straight on the
//! appkit main thread, so they work even while our window is hidden and egui isn't painting.

use super::TrayAction;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct TrayManager {
    tray: TrayIcon,
    _menu: Menu,
    _items: (MenuItem, MenuItem),
}

impl TrayManager {
    pub fn new(rgba: Vec<u8>, w: u32, h: u32) -> Option<TrayManager> {
        let icon = Icon::from_rgba(rgba, w, h).ok()?;
        let menu = Menu::new();
        let show = MenuItem::new("Show Citron v2", true, None);
        let quit = MenuItem::new("Quit Citron", true, None);
        menu.append_items(&[&show, &PredefinedMenuItem::separator(), &quit])
            .ok()?;
        let (show_id, quit_id) = (show.id().clone(), quit.id().clone());
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            if e.id == show_id {
                crate::os::show_app();
            } else if e.id == quit_id {
                crate::os::quit_now();
            }
        }));
        let tray = TrayIconBuilder::new()
            .with_tooltip("Citron v2")
            .with_icon(icon)
            .with_menu(Box::new(menu.clone()))
            .build()
            .ok()?;
        let _ = tray.set_visible(false);
        Some(TrayManager {
            tray,
            _menu: menu,
            _items: (show, quit),
        })
    }

    pub fn set_visible(&self, visible: bool) {
        let _ = self.tray.set_visible(visible);
    }

    /// everything is handled by the native menu (see new), nothing to poll
    pub fn poll(&self) -> Option<TrayAction> {
        None
    }
}
