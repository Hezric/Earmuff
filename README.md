# Earmuff

A quiet guard for sudden volume jumps.

[简体中文](README_zh-Hans.md)

Earmuff watches your default Windows output and brings unexpected loudness back
within reach. New devices start protected. A jump to full volume drops to **10%**;
other levels above **42%** are capped. Each device has its own protection switch.

A small tray flyout keeps your current output close by. Its settings hold the
limits, a pause of 1–720 minutes, and English or Simplified Chinese. The window
follows your Windows theme and accent, then releases its webview when dismissed.

## Build

Windows 11, Rust's MSVC toolchain with Visual Studio C++ Build Tools, Node.js LTS,
and Microsoft Edge WebView2 are required. No release has been published yet.

```powershell
git clone https://github.com/Hezric/Earmuff.git
cd Earmuff
npm ci
npm run tauri -- build --ci --no-bundle -- --locked
```

The portable executable is `src-tauri/target/release/earmuff.exe`. Launching it
turns protection on immediately. Earmuff leaves login startup to your startup
manager, such as Lineup.

### Use with an agent

Give this prompt to an agent that can build Windows software:

```text
Build Earmuff from https://github.com/Hezric/Earmuff in a local checkout.
Read its AGENTS.md. Check the Windows 11, MSVC Rust/C++ toolchain,
Node.js LTS, and WebView2 prerequisites. Run npm ci, then
npm run tauri -- build --ci --no-bundle -- --locked.
Verify the build produced src-tauri/target/release/earmuff.exe without
launching it. Explain that launching enables audio protection, and ask
before launching or configuring login startup.
```

## Use

Click the tray icon to open Earmuff; right-click for pause, resume, and quit.
The footer gear switches between devices and settings. Pause presets are 15,
30, 60, and 120 minutes; custom durations save on Enter or focus loss. Pausing
again uses the saved duration. Restarting Earmuff resumes protection.

Virtual outputs and display audio start under **Other devices**. Using one as
your default output moves it into the main list permanently. Turning a device's
switch off leaves its volume untouched, including at 100%.

Settings are stored in `%APPDATA%\Earmuff\Earmuff.ini`. If the file cannot be
read, Earmuff protects devices with safe defaults and shows the error. Changes
in the flyout are saved atomically.

## Preview and contribute

`npm run dev` opens a safe browser frontend at `http://127.0.0.1:1420` with
simulated devices. Preview Chinese with `?lang=zh` or English with `?lang=en`;
`&theme=light` and `&theme=dark` are preview-only controls.

[Contributing](CONTRIBUTING.md) · [Behavior and checks](docs/features/volume-guard.md)
· [MIT license](LICENSE) · [Third-party notices](THIRD-PARTY-NOTICES.md)

Earmuff limits Windows volume percentages, not measured sound pressure.
