<!-- #claude -->
# Citron v2 (macOS)

macOS-only fork of Citron Clicker, written in **Rust + [egui](https://github.com/emilk/egui)**.
It is an external autoclicker for Minecraft Java Edition.

![Citron v2](screenshot.png)

## Building

Requires a recent Rust toolchain and the Xcode command line tools (`xcode-select --install`).

```bash
cargo run --release
```

## Permissions

On first start macOS asks for two permissions. Grant them to your terminal (or the built app) under
System Settings > Privacy & Security:

- **Accessibility**: needed to send the clicks
- **Input Monitoring**: needed to read the real mouse buttons and hotkeys

## Notes

- F-keys need **Fn** held unless "Use F1, F2, etc. keys as standard function keys" is enabled in
  the keyboard settings. Rebind the panic key if that is annoying.
- Minecraft detection is name based (`java`, `minecraft`, Lunar, Badlion, ...).
- Tray icon, self-update, autostart and screen-capture hiding are not available on macOS.

## Credits

- UI framework: [egui / eframe](https://github.com/emilk/egui)
- Font: [Poppins](https://fonts.google.com/specimen/Poppins) (SIL Open Font License)
- Icons: [Lucide](https://lucide.dev) (ISC License)
- Default click sound: [Arsenic Client](https://github.com/ArsenicClient/Arsenic) (MIT License)
