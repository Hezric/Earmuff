# Earmuff

A quiet guard for sudden volume jumps.

[简体中文](README_zh-Hans.md)

Earmuff watches your default Windows output and brings unexpected loudness back
within reach. New devices start protected. A jump to full volume drops to **10%**;
other levels above **42%** are capped. Each device has its own protection switch.

A small tray flyout keeps your current output close by. Its settings hold the
limits, a pause of 1–720 minutes, and English or Simplified Chinese. The window
follows your Windows theme and accent, then releases its webview when dismissed.

## Install

With Scoop, run `scoop bucket add zephyr https://github.com/Hezric/ScoopZephyr`
and `scoop install zephyr/earmuff`. Use the executable under Scoop's `current`
directory for login startup so upgrades keep the same path.

Download the Windows x64 portable archive from [Releases](https://github.com/Hezric/Earmuff/releases/latest),
verify it against `SHA256SUMS`, and extract it. Keep WebView2 installed. Launching enables audio protection immediately.
Login startup remains with your startup manager.

## Build

Windows 11, Rust's MSVC toolchain with Visual Studio C++ Build Tools, Node.js LTS,
Microsoft Edge WebView2 and [Vite+](https://viteplus.dev/guide/) are required.

```powershell
#requires -Version 7.0
git clone https://github.com/Hezric/Earmuff.git
cd Earmuff
vp install --frozen-lockfile
vp exec tauri build --ci --no-bundle -- --locked
```

The portable executable is `src-tauri/target/release/earmuff.exe`. Launching it
turns protection on immediately. Earmuff leaves login startup to your startup
manager, such as Lineup.

### Use with an agent

Give this prompt to an agent that can build Windows software:

```text
Install Earmuff from https://github.com/Hezric/Earmuff/releases/latest.
Verify the Windows x64 archive against its SHA256SUMS, extract it, and check
that earmuff.exe exists without launching it. Preserve existing settings
and startup registration. Explain that launching enables volume protection; obtain authorization before launch.
Install skills/earmuff through my existing Skill management. If I have
none, ask first and recommend `npx skills add Hezric/Earmuff -g` using Vercel's
skills CLI (https://github.com/vercel-labs/skills; Node.js 22.20+).
Verify the installed Skill's path and complete contents too.
```

## Use

Click the tray icon to toggle Earmuff; right-click for pause, resume, and quit.
The footer gear switches between devices and settings. Pause presets are 15,
30, 60, and 120 minutes; custom durations save on Enter or focus loss. Pausing
again uses the saved duration. Restarting Earmuff resumes protection.

Virtual outputs and display audio stay under **Other devices**, including when
used as the default output. Turning a device's
switch off leaves its volume untouched, including at 100%.

Settings are stored in `%APPDATA%\Earmuff\Earmuff.ini`. If the file cannot be
read, Earmuff protects devices with safe defaults and shows the error. Changes
in the flyout are saved atomically.

A round check badge means protection is active; plain headphones mean the current
output is unprotected, and a round pause badge marks a pause. All states retain full contrast.
The native flyout uses Windows shadow and corners. Hidden icons open it in the
notification corner; outside clicks and Escape dismiss it.

## Preview and contribute

`vp run dev` opens a safe browser frontend at `http://127.0.0.1:1420` with
simulated devices. Preview Chinese with `?lang=zh` or English with `?lang=en`;
`&theme=light` and `&theme=dark` are preview-only controls.

[Contributing](CONTRIBUTING.md) · [Behavior and checks](docs/features/volume-guard.md)
· [MIT license](LICENSE) · [Third-party notices](THIRD-PARTY-NOTICES.md)

Earmuff limits Windows volume percentages, not measured sound pressure.
