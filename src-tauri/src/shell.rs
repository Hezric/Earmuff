use crate::core::State;
use crate::runtime::{Action, Message, Snapshot, Worker};
use tauri::{AppHandle, Manager};

include!("tray_assets.rs");

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    linen_tray::install(
        app,
        linen_tray::Config {
            id: "earmuff",
            title: "Earmuff",
            project: "https://github.com/Hezric/Earmuff",
            artwork: ARTWORK,
            initial: "protected",
        },
        |app| {
            if let Some(worker) = app.try_state::<Worker>() {
                let _ = worker
                    .sender
                    .send(Message::Request(Some(Action::TogglePause), None));
            }
        },
    )
}
#[derive(Debug, PartialEq)]
enum TrayState {
    Protected,
    Unprotected,
    Paused,
}
fn tray_state(state: &State) -> TrayState {
    if state.pause_remaining > 0 {
        TrayState::Paused
    } else if state
        .devices
        .iter()
        .any(|device| state.current.as_deref() == Some(device.id.as_str()) && device.protected)
    {
        TrayState::Protected
    } else {
        TrayState::Unprotected
    }
}

pub fn update_tray(app: &AppHandle, snapshot: &Snapshot) {
    let paused = snapshot.state.pause_remaining > 0;
    let zh = snapshot.appearance.language == "zh";
    let state = match tray_state(&snapshot.state) {
        TrayState::Protected => "protected",
        TrayState::Unprotected => "unprotected",
        TrayState::Paused => "paused",
    };
    let status = if snapshot.state.error.is_some() {
        if zh {
            "保护需要处理"
        } else {
            "Protection needs attention"
        }
    } else if paused {
        if zh {
            "保护已暂停"
        } else {
            "Protection paused"
        }
    } else if tray_state(&snapshot.state) == TrayState::Unprotected {
        if zh {
            "当前输出未受保护"
        } else {
            "Current output is not protected"
        }
    } else {
        if zh {
            "音量保护已开启"
        } else {
            "Volume protection is on"
        }
    };

    linen_tray::update_tray(
        app,
        &snapshot.appearance,
        state,
        paused,
        snapshot.state.pause_minutes,
        status,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{Device, Guard},
        settings::Settings,
    };
    #[test]
    fn icon_tracks_current_device_protection_and_pause() {
        let mut state = Guard::new(Settings::default()).state(0);
        assert_eq!(tray_state(&state), TrayState::Unprotected);
        state.devices = vec![
            Device {
                id: "usb".into(),
                name: "USB".into(),
                kind: "speaker".into(),
                protected: true,
                folded: false,
            },
            Device {
                id: "virtual".into(),
                name: "Virtual".into(),
                kind: "software".into(),
                protected: false,
                folded: true,
            },
        ];
        state.current = Some("usb".into());
        assert_eq!(tray_state(&state), TrayState::Protected);
        state.current = Some("virtual".into());
        assert_eq!(tray_state(&state), TrayState::Unprotected);
        state.devices[1].protected = true;
        assert_eq!(tray_state(&state), TrayState::Protected);
        for protected in [true, false] {
            state.devices[1].protected = protected;
            state.pause_remaining = 1;
            assert_eq!(tray_state(&state), TrayState::Paused);
        }
        state.pause_remaining = 0;
        assert_eq!(tray_state(&state), TrayState::Unprotected);
        state.current = Some("disconnected".into());
        assert_eq!(tray_state(&state), TrayState::Unprotected);
    }
}
