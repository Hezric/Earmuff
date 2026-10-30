use crate::{
    audio::WindowsAudio,
    core::{Guard, State},
    settings::Settings,
    shell,
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};
use windows::{
    core::w,
    Foundation::TypedEventHandler,
    Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
    UI::ViewManagement::UISettings,
};

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    SetLevels { cap: u8, drop: u8 },
    SetProtected { id: String, protected: bool },
    SetLanguage { language: String },
    SetPauseMinutes { minutes: u16 },
    TogglePause,
}
pub use linen_tray::Appearance;
use linen_tray::{read_taskbar_small, registry_light};
#[derive(Clone, Serialize)]
pub struct Snapshot {
    #[serde(flatten)]
    pub state: State,
    #[serde(flatten)]
    pub appearance: Appearance,
}
pub enum Message {
    DevicesChanged,
    VolumeChanged,
    AppearanceChanged,
    Request(Option<Action>, Option<Sender<Result<Snapshot, String>>>),
    Stop,
}
pub struct Worker {
    pub sender: Sender<Message>,
    pub thread: Mutex<Option<JoinHandle<()>>>,
}
impl Worker {
    pub fn request(&self, action: Option<Action>) -> Result<Snapshot, String> {
        let (send, receive) = mpsc::channel();
        self.sender
            .send(Message::Request(action, Some(send)))
            .map_err(|_| "Audio watcher is unavailable")?;
        receive
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "Audio watcher did not respond")?
    }
    pub fn stop(&self) {
        let _ = self.sender.send(Message::Stop);
        if let Some(thread) = self.thread.lock().ok().and_then(|mut t| t.take()) {
            let _ = thread.join();
        }
    }
}
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
pub fn start(app: AppHandle, path: PathBuf) -> Result<Worker, String> {
    let (sender, receive) = mpsc::channel();
    let thread_sender = sender.clone();
    let (ready_send, ready_receive) = mpsc::channel();
    let thread = std::thread::Builder::new()
        .name("earmuff-audio".into())
        .spawn(move || {
            let run = || -> Result<(), String> {
                unsafe {
                    CoInitializeEx(None, COINIT_MULTITHREADED)
                        .ok()
                        .map_err(|e| e.to_string())?;
                }
                let _apartment = Apartment;
                let mut audio =
                    WindowsAudio::new(thread_sender.clone()).map_err(|e| e.to_string())?;
                let colors = UISettings::new().map_err(|e| e.to_string())?;
                let appearance_sender = thread_sender.clone();
                let cookie = colors
                    .ColorValuesChanged(&TypedEventHandler::new(move |_, _| {
                        let _ = appearance_sender.send(Message::AppearanceChanged);
                        Ok(())
                    }))
                    .map_err(|e| e.to_string())?;
                let _ = ready_send.send(Ok(()));
                run_loop(&app, &path, receive, &mut audio, &colors);
                let _ = colors.RemoveColorValuesChanged(cookie);
                Ok(())
            };
            if let Err(error) = run() {
                let _ = ready_send.send(Err(error));
            }
        })
        .map_err(|e| e.to_string())?;
    match ready_receive.recv().map_err(|e| e.to_string())? {
        Ok(()) => Ok(Worker {
            sender,
            thread: Mutex::new(Some(thread)),
        }),
        Err(error) => {
            let _ = thread.join();
            Err(error)
        }
    }
}
fn run_loop(
    app: &AppHandle,
    path: &std::path::Path,
    receive: Receiver<Message>,
    audio: &mut WindowsAudio,
    colors: &UISettings,
) {
    let (settings, mut settings_error) = Settings::load(path);
    let mut guard = Guard::new(settings);
    let clock = Instant::now();
    let mut audio_error = None;
    let mut appearance = Appearance::read(colors);
    let mut next = Message::DevicesChanged;
    'worker: loop {
        let now = clock.elapsed().as_secs();
        let mut reply = None;
        let mut action_error = None;
        let mut persist = false;
        let mut enforce = false;
        let mut refresh_failed = false;
        match next {
            Message::Stop => break,
            Message::DevicesChanged => {
                let result = (|| -> Result<bool, String> {
                    let changed = guard.sync_default(audio)?;
                    // Protect the new default before enumerating names or writing settings.
                    guard.enforce(audio, now)?;
                    Ok(guard.refresh(audio)? || changed)
                })();
                match result {
                    Ok(changed) => {
                        persist = changed && settings_error.is_none();
                        audio_error = None;
                    }
                    Err(e) => {
                        audio_error = Some(e);
                        refresh_failed = true;
                    }
                }
                enforce = true;
            }
            Message::VolumeChanged => enforce = true,
            Message::AppearanceChanged => appearance = Appearance::read(colors),
            Message::Request(action, response) => {
                reply = response;
                if let Some(action) = action {
                    let should_save = !matches!(&action, Action::TogglePause);
                    let result = match action {
                        Action::SetLanguage { language } => {
                            if ["system", "zh", "en"].contains(&language.as_str()) {
                                guard.settings.language = language;
                                Ok(())
                            } else {
                                Err("Invalid language".into())
                            }
                        }
                        Action::SetPauseMinutes { minutes } => guard.set_pause_minutes(minutes),
                        Action::SetLevels { cap, drop } => guard.set_levels(cap, drop),
                        Action::SetProtected { id, protected } => {
                            guard.set_protected(&id, protected)
                        }
                        Action::TogglePause => {
                            if guard.state(now).pause_remaining > 0 {
                                guard.resume();
                            } else {
                                guard.pause(now);
                            }
                            Ok(())
                        }
                    };
                    match result {
                        Ok(()) => {
                            persist = should_save;
                            enforce = true;
                        }
                        Err(e) => action_error = Some(e),
                    }
                }
            }
        }
        if enforce {
            let result = guard.enforce(audio, now);
            if !refresh_failed || result.is_err() {
                audio_error = result.err();
            }
        }
        if persist {
            settings_error = guard.settings.save(path).err();
        }
        let mut state = guard.state(now);
        state.error = audio_error.clone().or(settings_error.clone());
        let mut resolved_appearance = appearance.clone();
        if guard.settings.language != "system" {
            resolved_appearance.language = guard.settings.language.clone();
        }
        let snapshot = Snapshot {
            state,
            appearance: resolved_appearance,
        };
        let tray_app = app.clone();
        let tray_snapshot = snapshot.clone();
        let _ = app.run_on_main_thread(move || shell::update_tray(&tray_app, &tray_snapshot));
        let _ = app.emit("state-changed", &snapshot);
        let idle_snapshot = snapshot.clone();
        if let Some(reply) = reply {
            let _ = reply.send(action_error.map_or_else(|| Ok(snapshot), Err));
        }
        let remaining = guard.state(clock.elapsed().as_secs()).pause_remaining;
        next = loop {
            match receive.recv_timeout(Duration::from_secs(1)) {
                Ok(message) => break message,
                Err(mpsc::RecvTimeoutError::Disconnected) => break 'worker,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if remaining > 0 && guard.state(clock.elapsed().as_secs()).pause_remaining == 0
                    {
                        break Message::VolumeChanged;
                    }
                    // Taskbar size and DPI changes need no audio work or resident webview.
                    let mut snapshot = idle_snapshot.clone();
                    snapshot.appearance.taskbar_small = read_taskbar_small();
                    snapshot.appearance.taskbar_light =
                        registry_light(w!("SystemUsesLightTheme")) != 0;
                    appearance.taskbar_small = snapshot.appearance.taskbar_small;
                    appearance.taskbar_light = snapshot.appearance.taskbar_light;
                    let app = app.clone();
                    let _ = app
                        .clone()
                        .run_on_main_thread(move || shell::update_tray(&app, &snapshot));
                }
            }
        };
    }
}
