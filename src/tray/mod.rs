// #claude
//! menu bar icon (macos), see `mac_claude`.

mod mac_claude;
pub use mac_claude::*;

pub enum TrayAction {
    Show,
    /// right-click: open our themed menu at this screen position (physical px)
    Menu { x: f64, y: f64 },
}
