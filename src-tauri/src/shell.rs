use crate::{
    core::State,
    runtime::{Action, Message, Snapshot, Worker},
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

struct TrayItems {
    open: MenuItem<tauri::Wry>,
    pause: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let zh = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() & 0x3ff == 4 };
    let open = MenuItem::with_id(
        app,
        "open",
        if zh { "打开 Earmuff" } else { "Open Earmuff" },
        true,
        None::<&str>,
    )?;
    let pause = MenuItem::with_id(
        app,
        "pause",
        if zh {
            "暂停 1 小时"
        } else {
            "Pause for 1 hour"
        },
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        if zh { "退出" } else { "Quit" },
        true,
        None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[&open, &pause, &quit])?;
    app.manage(TrayItems { open, pause, quit });
    TrayIconBuilder::with_id("earmuff")
        .icon(Image::from_bytes(include_bytes!(
            "../icons/dark/32x32.png"
        ))?)
        .tooltip("Earmuff")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => open_flyout(app),
            "pause" => {
                if let Some(worker) = app.try_state::<Worker>() {
                    let _ = worker
                        .sender
                        .send(Message::Request(Some(Action::TogglePause), None));
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                open_flyout(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
pub fn open_flyout(app: &AppHandle) {
    let app = app.clone();
    // WebView2 creation deadlocks inside synchronous Windows event handlers.
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = build_flyout(&app) {
            eprintln!("{error}");
        }
    });
}
fn build_flyout(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("flyout") {
        window.set_focus()?;
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(app, "flyout", WebviewUrl::App("index.html".into()))
        .title("Earmuff")
        .inner_size(360.0, 464.0)
        .min_inner_size(300.0, 200.0)
        .background_color(tauri::window::Color(0, 0, 0, 0))
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .visible(false)
        .build()?;
    if let Ok(hwnd) = window.hwnd() {
        use windows::Win32::{Foundation::HWND, Graphics::Dwm::*, UI::WindowsAndMessaging::*};
        unsafe {
            let hwnd = HWND(hwnd.0);
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                (style | WS_EX_TOOLWINDOW.0 as isize) & !(WS_EX_APPWINDOW.0 as isize),
            );
            // CSS owns the visible panel; DWM must not outline the full transparent viewport.
            let policy = DWMNCRP_DISABLED;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_NCRENDERING_POLICY,
                (&policy as *const DWMNCRENDERINGPOLICY).cast(),
                std::mem::size_of_val(&policy) as u32,
            );
        }
    }
    Ok(())
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlyoutLayout {
    limit: f64,
    anchor_bottom: f64,
    top_anchored: bool,
}
#[tauri::command]
pub async fn resize_flyout(app: AppHandle, height: f64) -> Result<FlyoutLayout, String> {
    tauri::async_runtime::spawn_blocking(move || resize_window(app, height))
        .await
        .map_err(|e| e.to_string())?
}
fn resize_window(app: AppHandle, height: f64) -> Result<FlyoutLayout, String> {
    if !height.is_finite() {
        return Err("Invalid flyout height".into());
    }
    let window = app
        .get_webview_window("flyout")
        .ok_or("Flyout was closed")?;
    let anchor = app
        .tray_by_id("earmuff")
        .and_then(|tray| tray.rect().ok().flatten());
    let monitor = match &anchor {
        Some(rect) => {
            let point = rect.position.to_physical::<i32>(1.0);
            app.monitor_from_point(point.x as f64, point.y as f64)
        }
        None => app.primary_monitor(),
    }
    .map_err(|e| e.to_string())?
    .ok_or("No monitor is available")?;
    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    let viewport_height = work.size.height;
    let gap = (12.0 * scale).round() as i32;
    let max_height = work
        .size
        .height
        .saturating_sub(2 * (12.0 * scale).round() as u32) as f64
        / scale;
    let work_rect = [
        work.position.x,
        work.position.y,
        work.position.x + work.size.width as i32,
        work.position.y + work.size.height as i32,
    ];
    let anchor_rect = anchor
        .map(|r| {
            let p = r.position.to_physical::<i32>(scale);
            let s = r.size.to_physical::<u32>(scale);
            [p.x, p.y, p.x + s.width as i32, p.y + s.height as i32]
        })
        .unwrap_or([work_rect[2], work_rect[3], work_rect[2], work_rect[3]]);
    let (x, y, width, panel_height) = placement(work_rect, anchor_rect, scale, height);
    let full = placement(work_rect, anchor_rect, scale, max_height);
    let edge = taskbar_edge(work_rect, anchor_rect);
    let anchor_bottom = if edge == 2 {
        work_rect[3] - gap
    } else {
        anchor_rect[3]
    };
    let layout = FlyoutLayout {
        limit: max_height,
        anchor_bottom: (anchor_bottom - work_rect[1]) as f64 / scale,
        top_anchored: edge == 0,
    };
    // Windows retains an invisible resize frame around the undecorated client area.
    let outer = window.outer_size().map_err(|e| e.to_string())?;
    let inner = window.inner_size().map_err(|e| e.to_string())?;
    let frame_width = outer.width.saturating_sub(inner.width);
    let frame_height = outer.height.saturating_sub(inner.height);
    use windows::Win32::{Foundation::HWND, Graphics::Gdi::*, UI::WindowsAndMessaging::*};
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    let (send, receive) = std::sync::mpsc::channel();
    window
        .with_webview(move |_| {
            let result = (|| -> Result<(), String> {
                unsafe {
                    // Keep WebView2's viewport stable; per-frame resizes race its compositor.
                    SetWindowPos(
                        HWND(hwnd as *mut _),
                        None,
                        full.0 - gap - (frame_width / 2) as i32,
                        work_rect[1],
                        width as i32 + 2 * gap + frame_width as i32,
                        viewport_height as i32 + frame_height as i32,
                        SWP_NOACTIVATE | SWP_NOZORDER,
                    )
                    .map_err(|e| e.to_string())?;
                    // Clip drawing and hit testing, leaving room for the panel's CSS shadow.
                    let left = (x - full.0).max(0);
                    let top = (y - work_rect[1] - gap).max(0);
                    let region = CreateRectRgn(
                        left,
                        top,
                        left + width as i32 + 2 * gap + frame_width as i32,
                        (y - work_rect[1] + panel_height as i32 + gap).min(viewport_height as i32),
                    );
                    if region.0.is_null() {
                        return Err("Could not create flyout region".into());
                    }
                    if SetWindowRgn(HWND(hwnd as *mut _), Some(region), true) == 0 {
                        let _ = DeleteObject(region.into());
                        return Err("Could not apply flyout region".into());
                    }
                }
                Ok(())
            })();
            let _ = send.send(result);
        })
        .map_err(|e| e.to_string())?;
    receive
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| "Flyout resize did not respond")??;
    Ok(layout)
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
fn tray_icon(icon_state: &TrayState, light: bool, size: u32, small: bool) -> &'static [u8] {
    let icons: [&[u8]; 2] = match (icon_state, light, size) {
        (TrayState::Protected, true, 16) => [
            include_bytes!("../icons/light/16x16.png"),
            include_bytes!("../icons/small-light/16x16.png"),
        ],
        (TrayState::Protected, false, 16) => [
            include_bytes!("../icons/dark/16x16.png"),
            include_bytes!("../icons/small-dark/16x16.png"),
        ],
        (TrayState::Unprotected, true, 16) => [
            include_bytes!("../icons/unprotected-light/16x16.png"),
            include_bytes!("../icons/small-unprotected-light/16x16.png"),
        ],
        (TrayState::Unprotected, false, 16) => [
            include_bytes!("../icons/unprotected-dark/16x16.png"),
            include_bytes!("../icons/small-unprotected-dark/16x16.png"),
        ],
        (TrayState::Paused, true, 16) => [
            include_bytes!("../icons/paused-light/16x16.png"),
            include_bytes!("../icons/small-paused-light/16x16.png"),
        ],
        (TrayState::Paused, false, 16) => [
            include_bytes!("../icons/paused-dark/16x16.png"),
            include_bytes!("../icons/small-paused-dark/16x16.png"),
        ],
        (TrayState::Protected, true, 20) => [
            include_bytes!("../icons/light/20x20.png"),
            include_bytes!("../icons/small-light/20x20.png"),
        ],
        (TrayState::Protected, false, 20) => [
            include_bytes!("../icons/dark/20x20.png"),
            include_bytes!("../icons/small-dark/20x20.png"),
        ],
        (TrayState::Unprotected, true, 20) => [
            include_bytes!("../icons/unprotected-light/20x20.png"),
            include_bytes!("../icons/small-unprotected-light/20x20.png"),
        ],
        (TrayState::Unprotected, false, 20) => [
            include_bytes!("../icons/unprotected-dark/20x20.png"),
            include_bytes!("../icons/small-unprotected-dark/20x20.png"),
        ],
        (TrayState::Paused, true, 20) => [
            include_bytes!("../icons/paused-light/20x20.png"),
            include_bytes!("../icons/small-paused-light/20x20.png"),
        ],
        (TrayState::Paused, false, 20) => [
            include_bytes!("../icons/paused-dark/20x20.png"),
            include_bytes!("../icons/small-paused-dark/20x20.png"),
        ],
        (TrayState::Protected, true, 24) => [
            include_bytes!("../icons/light/24x24.png"),
            include_bytes!("../icons/small-light/24x24.png"),
        ],
        (TrayState::Protected, false, 24) => [
            include_bytes!("../icons/dark/24x24.png"),
            include_bytes!("../icons/small-dark/24x24.png"),
        ],
        (TrayState::Unprotected, true, 24) => [
            include_bytes!("../icons/unprotected-light/24x24.png"),
            include_bytes!("../icons/small-unprotected-light/24x24.png"),
        ],
        (TrayState::Unprotected, false, 24) => [
            include_bytes!("../icons/unprotected-dark/24x24.png"),
            include_bytes!("../icons/small-unprotected-dark/24x24.png"),
        ],
        (TrayState::Paused, true, 24) => [
            include_bytes!("../icons/paused-light/24x24.png"),
            include_bytes!("../icons/small-paused-light/24x24.png"),
        ],
        (TrayState::Paused, false, 24) => [
            include_bytes!("../icons/paused-dark/24x24.png"),
            include_bytes!("../icons/small-paused-dark/24x24.png"),
        ],
        (TrayState::Protected, true, 32) => [
            include_bytes!("../icons/light/32x32.png"),
            include_bytes!("../icons/small-light/32x32.png"),
        ],
        (TrayState::Protected, false, 32) => [
            include_bytes!("../icons/dark/32x32.png"),
            include_bytes!("../icons/small-dark/32x32.png"),
        ],
        (TrayState::Unprotected, true, 32) => [
            include_bytes!("../icons/unprotected-light/32x32.png"),
            include_bytes!("../icons/small-unprotected-light/32x32.png"),
        ],
        (TrayState::Unprotected, false, 32) => [
            include_bytes!("../icons/unprotected-dark/32x32.png"),
            include_bytes!("../icons/small-unprotected-dark/32x32.png"),
        ],
        (TrayState::Paused, true, 32) => [
            include_bytes!("../icons/paused-light/32x32.png"),
            include_bytes!("../icons/small-paused-light/32x32.png"),
        ],
        (TrayState::Paused, false, 32) => [
            include_bytes!("../icons/paused-dark/32x32.png"),
            include_bytes!("../icons/small-paused-dark/32x32.png"),
        ],
        (TrayState::Protected, true, 48) => [
            include_bytes!("../icons/light/48x48.png"),
            include_bytes!("../icons/small-light/48x48.png"),
        ],
        (TrayState::Protected, false, 48) => [
            include_bytes!("../icons/dark/48x48.png"),
            include_bytes!("../icons/small-dark/48x48.png"),
        ],
        (TrayState::Unprotected, true, 48) => [
            include_bytes!("../icons/unprotected-light/48x48.png"),
            include_bytes!("../icons/small-unprotected-light/48x48.png"),
        ],
        (TrayState::Unprotected, false, 48) => [
            include_bytes!("../icons/unprotected-dark/48x48.png"),
            include_bytes!("../icons/small-unprotected-dark/48x48.png"),
        ],
        (TrayState::Paused, true, 48) => [
            include_bytes!("../icons/paused-light/48x48.png"),
            include_bytes!("../icons/small-paused-light/48x48.png"),
        ],
        (TrayState::Paused, false, 48) => [
            include_bytes!("../icons/paused-dark/48x48.png"),
            include_bytes!("../icons/small-paused-dark/48x48.png"),
        ],
        _ => [
            include_bytes!("../icons/dark/32x32.png"),
            include_bytes!("../icons/small-dark/32x32.png"),
        ],
    };
    icons[usize::from(small)]
}
pub fn update_tray(app: &AppHandle, snapshot: &Snapshot) {
    let paused = snapshot.state.pause_remaining > 0;
    let light = snapshot.appearance.taskbar_light;
    let scale = app
        .tray_by_id("earmuff")
        .and_then(|t| t.rect().ok().flatten())
        .and_then(|r| {
            let p = r.position.to_physical::<i32>(1.0);
            app.monitor_from_point(p.x as f64, p.y as f64)
                .ok()
                .flatten()
        })
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);
    let size = if scale < 1.125 {
        16
    } else if scale < 1.375 {
        20
    } else if scale < 1.75 {
        24
    } else if scale < 2.5 {
        32
    } else {
        48
    };
    let icon_state = tray_state(&snapshot.state);
    let data = tray_icon(&icon_state, light, size, snapshot.appearance.taskbar_small);
    let zh = snapshot.appearance.language == "zh";
    let minutes = snapshot.state.pause_minutes;
    let duration = if minutes.is_multiple_of(60) {
        if zh {
            format!("暂停 {} 小时", minutes / 60)
        } else {
            format!("Pause {} hr", minutes / 60)
        }
    } else if zh {
        format!("暂停 {minutes} 分钟")
    } else {
        format!("Pause {minutes} min")
    };
    let label = if paused {
        if zh {
            "恢复保护"
        } else {
            "Resume now"
        }
    } else {
        &duration
    };
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.pause.set_text(label);
        let _ = items
            .open
            .set_text(if zh { "打开 Earmuff" } else { "Open Earmuff" });
        let _ = items.quit.set_text(if zh { "退出" } else { "Quit" });
    }
    if let Some(tray) = app.tray_by_id("earmuff") {
        if let Ok(icon) = Image::from_bytes(data) {
            let _ = tray.set_icon(Some(icon));
        }
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
        } else if icon_state == TrayState::Unprotected {
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
        let _ = tray.set_tooltip(Some(format!("Earmuff — {status}")));
    }
}

fn taskbar_edge(work: [i32; 4], anchor: [i32; 4]) -> usize {
    let cx = (anchor[0] + anchor[2]) / 2;
    let cy = (anchor[1] + anchor[3]) / 2;
    [
        (cy - work[1]).abs(),
        (cx - work[2]).abs(),
        (cy - work[3]).abs(),
        (cx - work[0]).abs(),
    ]
    .iter()
    .enumerate()
    .min_by_key(|(_, distance)| *distance)
    .map(|(i, _)| i)
    .unwrap_or(2)
}
fn placement(work: [i32; 4], anchor: [i32; 4], scale: f64, height: f64) -> (i32, i32, u32, u32) {
    let gap = (12.0 * scale).round() as i32;
    let width = (360.0 * scale)
        .round()
        .min((work[2] - work[0] - 2 * gap).max(1) as f64) as i32;
    let height = (height * scale)
        .round()
        .min((work[3] - work[1] - 2 * gap).max(1) as f64)
        .max(1.0) as i32;
    let cx = (anchor[0] + anchor[2]) / 2;
    let edge = taskbar_edge(work, anchor);
    let mut x = cx - width / 2;
    let mut y = anchor[3] - height;
    match edge {
        0 => y = work[1] + gap,
        1 => x = work[2] - width - gap,
        2 => y = work[3] - height - gap,
        _ => x = work[0] + gap,
    }
    (
        x.clamp(work[0] + gap, (work[2] - width - gap).max(work[0] + gap)),
        y.clamp(work[1] + gap, (work[3] - height - gap).max(work[1] + gap)),
        width as u32,
        height as u32,
    )
}
fn animations_enabled() -> bool {
    use windows::{core::BOOL, Win32::UI::WindowsAndMessaging::*};
    let mut enabled = BOOL(0);
    unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&mut enabled as *mut BOOL).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_ok()
            && enabled.as_bool()
    }
}
fn animate_flyout(
    window: &WebviewWindow,
    show: bool,
    prepared_style: Option<isize>,
) -> Result<(), String> {
    use std::time::{Duration, Instant};
    use windows::Win32::{
        Foundation::{COLORREF, HWND, RECT},
        UI::WindowsAndMessaging::*,
    };
    let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
    unsafe {
        let mut bounds = RECT::default();
        GetWindowRect(hwnd, &mut bounds).map_err(|e| e.to_string())?;
        let current_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let style = prepared_style.unwrap_or(current_style);
        if prepared_style.is_some() && style | WS_EX_LAYERED.0 as isize != current_style {
            return Err("Flyout style changed before presentation".into());
        }
        if !animations_enabled() {
            if show {
                SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)
                    .map_err(|e| e.to_string())?;
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style);
                window.show().map_err(|e| e.to_string())?;
            }
            return Ok(());
        }
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED.0 as isize);
        let result = (|| {
            SetLayeredWindowAttributes(hwnd, COLORREF(0), if show { 0 } else { 255 }, LWA_ALPHA)?;
            if show {
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            }
            let start = Instant::now();
            loop {
                let t = if animations_enabled() {
                    (start.elapsed().as_secs_f64() / 0.2).min(1.0)
                } else {
                    1.0
                };
                let eased = 1.0 - (1.0 - t).powi(3);
                let visible = if show { eased } else { 1.0 - eased };
                SetLayeredWindowAttributes(
                    hwnd,
                    COLORREF(0),
                    (255.0 * visible).round() as u8,
                    LWA_ALPHA,
                )?;
                SetWindowPos(
                    hwnd,
                    None,
                    bounds.left,
                    bounds.top + (12.0 * (1.0 - visible)).round() as i32,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER,
                )?;
                if t >= 1.0 {
                    break;
                }
                std::thread::sleep(Duration::from_millis(8));
            }
            Ok::<_, windows::core::Error>(())
        })();
        // Restore normal composition before retaining or destroying the webview.
        if !show {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style);
        let _ = SetWindowPos(
            hwnd,
            None,
            bounds.left,
            bounds.top,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER,
        );
        result.map_err(|e| e.to_string())
    }
}
#[tauri::command]
pub async fn prepare_flyout(app: AppHandle, height: f64) -> Result<isize, String> {
    tauri::async_runtime::spawn_blocking(move || {
        resize_window(app.clone(), height)?;
        let window = app
            .get_webview_window("flyout")
            .ok_or("Flyout was closed")?;
        use windows::Win32::{
            Foundation::{COLORREF, HWND},
            UI::WindowsAndMessaging::*,
        };
        unsafe {
            let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED.0 as isize);
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA)
                .map_err(|e| e.to_string())?;
            // WebView2 needs visible renderer frames before the first nontransparent native frame.
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            Ok(style)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn present_flyout(app: AppHandle, original_style: isize) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(window) = app.get_webview_window("flyout") {
            animate_flyout(&window, true, Some(original_style))?;
            window.set_focus().map_err(|e| e.to_string())?;
            // Initialization can emit focus loss before presentation; dismissal starts afterward.
            let closing = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let handle = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                }
                if matches!(
                    event,
                    tauri::WindowEvent::Focused(false) | tauri::WindowEvent::CloseRequested { .. }
                ) && !closing.swap(true, std::sync::atomic::Ordering::Relaxed)
                {
                    let handle = handle.clone();
                    tauri::async_runtime::spawn_blocking(move || {
                        if let Err(error) = animate_flyout(&handle, false, None) {
                            eprintln!("{error}");
                        }
                        let _ = handle.destroy();
                    });
                }
            });
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn dismiss_flyout(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("flyout") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
pub fn open_project() -> Result<(), String> {
    use windows::{
        core::w,
        Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    };
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            w!("https://github.com/Hezric/Earmuff"),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        Err("Could not open the project page".into())
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod placement_tests {
    use super::{placement, tray_icon, tray_state, TrayState};
    use crate::{
        core::{Device, Guard},
        settings::Settings,
    };
    #[test]
    fn small_taskbar_uses_inset_artwork_in_the_same_dpi_scaled_image_box() {
        for (size, small_extent) in [(16, 12), (20, 15), (24, 18)] {
            for state in [
                TrayState::Protected,
                TrayState::Unprotected,
                TrayState::Paused,
            ] {
                for light in [true, false] {
                    let normal =
                        tauri::image::Image::from_bytes(tray_icon(&state, light, size, false))
                            .unwrap();
                    let small =
                        tauri::image::Image::from_bytes(tray_icon(&state, light, size, true))
                            .unwrap();
                    assert_eq!(
                        (normal.width(), small.width(), small.height()),
                        (size, size, size)
                    );
                    let ink_width = |image: &tauri::image::Image<'_>| {
                        let columns: Vec<_> = image
                            .rgba()
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .enumerate()
                            .filter(|(_, pixel)| pixel[3] > 32)
                            .map(|(index, _)| index as u32 % size)
                            .collect();
                        columns.iter().max().unwrap() - columns.iter().min().unwrap() + 1
                    };
                    // The user's 125% Wi-Fi captures span 20px Default and 15px Small.
                    assert_eq!(ink_width(&normal), size);
                    assert_eq!(ink_width(&small), small_extent);
                }
            }
        }
    }
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
    #[test]
    fn centers_and_clamps_at_both_edges_at_100_and_150_percent() {
        for (scale, work, center, left, right, expected_y, width, height) in [
            (
                1.0,
                [0, 0, 1920, 1040],
                [950, 1040, 970, 1080],
                [0, 1040, 20, 1080],
                [1900, 1040, 1920, 1080],
                628,
                360,
                400,
            ),
            (
                1.5,
                [-2880, 0, 0, 1560],
                [-1455, 1560, -1425, 1620],
                [-2880, 1560, -2850, 1620],
                [-30, 1560, 0, 1620],
                942,
                540,
                600,
            ),
        ] {
            let gap = (12.0_f64 * scale).round() as i32;
            assert_eq!(
                placement(work, center, scale, 400.0),
                (
                    (center[0] + center[2] - width as i32) / 2,
                    expected_y,
                    width,
                    height
                )
            );
            assert_eq!(placement(work, left, scale, 400.0).0, work[0] + gap);
            assert_eq!(
                placement(work, right, scale, 400.0).0,
                work[2] - gap - width as i32
            );
        }
    }
    #[test]
    fn anchors_at_each_taskbar_edge_with_scaled_gap_on_negative_monitor() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let gap = (12.0 * scale) as i32;
            let work = [-2000, 100, -400, 1000];
            for (anchor, edge) in [
                ([-600, 1000, -580, 1020], "bottom"),
                ([-600, 80, -580, 100], "top"),
                ([-2020, 700, -2000, 720], "left"),
                ([-400, 700, -380, 720], "right"),
            ] {
                let (x, y, width, height) = placement(work, anchor, scale, 400.0);
                assert_eq!(width, (360.0 * scale) as u32);
                assert_eq!(height, (400.0 * scale) as u32);
                assert!(
                    x >= work[0] + gap
                        && y >= work[1] + gap
                        && x + width as i32 <= work[2] - gap
                        && y + height as i32 <= work[3] - gap
                );
                match edge {
                    "bottom" => assert_eq!(y + height as i32, work[3] - gap),
                    "top" => assert_eq!(y, work[1] + gap),
                    "left" => assert_eq!(x, work[0] + gap),
                    _ => assert_eq!(x + width as i32, work[2] - gap),
                }
            }
        }
    }
}
