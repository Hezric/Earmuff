---
name: earmuff
description: Install, inspect or configure Earmuff on Windows through its native tray UI.
---

# Earmuff

Download the Windows x64 portable archive from the matching signed release in `Hezric/Earmuff`. Verify its SHA256 against that release's `SHA256SUMS` before extracting. Keep an existing installation's settings and login registration intact during an update. The archive keeps license notices under `licenses/`. Install this Skill separately from the repository.

Launching enables volume protection immediately. Obtain authorization before launching or configuring protection on real outputs. Never raise a real speaker or headset volume for a check; use simulated tests or browser preview.

Settings live in `%APPDATA%\Earmuff\Earmuff.toml`. The flyout gear holds the cap, full-volume fallback, pause duration and language. Each output has its own protection switch. Unprotected outputs remain untouched. New outputs start protected. Verify requested settings in the UI without changing the Windows default output or startup registration.

Left-click toggles the flyout; outside clicks and Escape dismiss it. Every icon opens the flyout from the right at the bottom-right corner of the current screen; height changes preserve the footer position. Right-click offers Open, Pause/Resume and Quit. Installation alone does not authorize a live hardware test.
