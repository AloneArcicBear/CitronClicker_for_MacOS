<!-- #claude -->
# Citron for macOS

An external autoclicker for **Minecraft Java Edition** on **macOS**, written in Rust with
[egui](https://github.com/emilk/egui).

This is a macOS port of [Citron v2 by center2055](https://github.com/center2055/CitronClicker),
which was Windows-only. All credit for the original app, its design and the clicker engine goes to
the original author.

![Citron](screenshot.png)

> **Made with AI.** The macOS port was written with **Claude** (an AI assistant by Anthropic) in
> one session, guided and tested by me on a real Mac. The code was reviewed and run, but it has not
> been audited. Use it at your own risk.

## What changed compared to the original

The original app only clicked on Windows. On macOS it started, but all OS functions were empty
placeholders, so it never clicked.

- **New macOS input backend** (`src/os/mac_claude.rs`)
  - clicks are sent with Quartz event services (`CGEventPost`)
  - the real mouse buttons and hotkeys are read from the hardware state, so the app never confuses
    its own clicks with yours
  - Minecraft detection uses the name of the frontmost app (`java`, `minecraft`, Lunar, Badlion …)
  - Windows key codes from the original are mapped to macOS key codes
- **Native macOS window**: real traffic-light buttons, native rounded corners and shadow; the
  traffic lights, logo and status pill are centred on one line
- **Settings rebuilt for macOS**
  - *Start at login* uses a LaunchAgent in `~/Library/LaunchAgents`
  - *Close to menu bar* shows a lemon icon in the menu bar with *Show* and *Quit*
  - *Hide from capture* hides the window from screenshots and screen recordings
  - *Dock hide key* hides the Dock icon with a hotkey, even while in-game
- **Removed**
  - all Windows code (Win32 backend, Windows tray, auto-updater, icon embedding, Windows CI)
  - Minecraft **Bedrock** support (Bedrock doesn't exist on macOS)
  - *Auto-update* (there are no macOS releases to update from)
- **Bug fixes**
  - the panic key could be bound to a mouse button, which quit the app on every click. Mouse
    buttons are now refused, and old saved bindings are reset to F8
  - compiler warnings about float literals
- **App bundle**: `make_app_claude.sh` builds a proper `Citron.app` with its own icon

Files and code written by the AI are marked with `_claude` in the file name or a `#claude` comment
in the first line.

## Install

Download `Citron-macOS.zip` from the [Releases](../../releases) page, unzip it and drag
`Citron.app` into your Applications folder.

The app is not notarized by Apple, so macOS blocks it the first time. Right-click the app and
choose **Open**, or click **Open Anyway** under System Settings > Privacy & Security.

Requires a Mac with Apple Silicon (M1 or newer).

## Permissions

On first start macOS asks for two permissions. Grant both to **Citron** under
System Settings > Privacy & Security:

- **Accessibility**: needed to send the clicks
- **Input Monitoring**: needed to read the real mouse buttons and hotkeys

If you rebuild the app yourself, macOS may revoke them. Turn the switch for Citron off and on again.

## Build it yourself

You need a Rust toolchain ([rustup.rs](https://rustup.rs)) and the Xcode command line tools
(`xcode-select --install`).

```bash
cargo run --release        # run directly from the terminal
bash make_app_claude.sh    # build Citron.app
```

## Notes

- F-keys need **Fn** held unless "Use F1, F2, etc. keys as standard function keys" is enabled in
  the keyboard settings. Rebind the panic key if that is annoying.
- Minecraft detection is name based, so another Java program in the foreground also counts as
  Minecraft.
- *Hide from capture* is not respected by every screen recording app on newer macOS versions.

## Credits

- Original app: [Citron v2 by center2055](https://github.com/center2055/CitronClicker)
- macOS port: written with Claude (Anthropic)
- UI framework: [egui / eframe](https://github.com/emilk/egui)
- Font: [Poppins](https://fonts.google.com/specimen/Poppins) (SIL Open Font License)
- Icons: [Lucide](https://lucide.dev) (ISC License)
- Default click sound: [Arsenic Client](https://github.com/ArsenicClient/Arsenic) (MIT License)
