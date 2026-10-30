use earmuff::{
    core::{AudioBackend, Endpoint, Guard},
    settings::Settings,
};
#[derive(Default)]
struct SimulatedAudio {
    endpoints: Vec<Endpoint>,
    current: Option<String>,
    volume: f32,
    writes: Vec<(String, f32)>,
    fail: bool,
}
impl SimulatedAudio {
    fn speaker(volume: f32) -> Self {
        Self {
            endpoints: vec![endpoint("usb", false, false)],
            current: Some("usb".into()),
            volume,
            ..Self::default()
        }
    }
}
fn endpoint(id: &str, software: bool, display: bool) -> Endpoint {
    Endpoint {
        id: id.into(),
        name: format!("Device {id}"),
        software,
        display,
        headphones: false,
    }
}
impl AudioBackend for SimulatedAudio {
    fn active_devices(&mut self) -> Result<Vec<Endpoint>, String> {
        Ok(self.endpoints.clone())
    }
    fn default_output(&mut self) -> Result<Option<String>, String> {
        Ok(self.current.clone())
    }
    fn volume(&mut self, _: &str) -> Result<f32, String> {
        if self.fail {
            Err("Disconnected".into())
        } else {
            Ok(self.volume)
        }
    }
    fn set_volume(&mut self, id: &str, percent: f32) -> Result<(), String> {
        self.writes.push((id.into(), percent));
        self.volume = percent;
        Ok(())
    }
}
#[test]
fn protection_through_audio_boundary() {
    for (volume, protected, expected) in [
        (100.0, true, Some(10.0)),
        (85.0, true, Some(42.0)),
        (42.1, true, Some(42.0)),
        (42.0, true, None),
        (0.0, true, None),
        (100.0, false, None),
        (85.0, false, None),
    ] {
        let mut audio = SimulatedAudio::speaker(volume);
        let mut guard = Guard::new(Settings::default());
        guard.refresh(&mut audio).unwrap();
        guard.set_protected("usb", protected).unwrap();
        guard.enforce(&mut audio, 0).unwrap();
        assert_eq!(
            audio.writes.last().map(|(_, v)| *v),
            expected,
            "{volume}, protected={protected}"
        );
    }
}
#[test]
fn discovery_folds_by_class_even_after_default_use_and_restart() {
    let mut audio = SimulatedAudio {
        endpoints: vec![
            endpoint("usb", false, false),
            endpoint("virtual", true, false),
            endpoint("display", false, true),
        ],
        ..SimulatedAudio::default()
    };
    let mut guard = Guard::new(Settings::default());
    assert!(guard.refresh(&mut audio).unwrap());
    let state = guard.state(0);
    assert!(state.devices.iter().all(|d| d.protected));
    assert_eq!(
        state.devices.iter().map(|d| d.folded).collect::<Vec<_>>(),
        [false, true, true]
    );
    audio.current = Some("virtual".into());
    assert!(guard.refresh(&mut audio).unwrap());
    audio.current = Some("usb".into());
    guard.refresh(&mut audio).unwrap();
    assert!(
        guard
            .state(0)
            .devices
            .iter()
            .find(|d| d.id == "virtual")
            .unwrap()
            .folded
    );
    let saved = guard.settings.clone();
    let mut restarted = Guard::new(saved);
    restarted.refresh(&mut audio).unwrap();
    assert!(
        restarted
            .state(0)
            .devices
            .iter()
            .find(|d| d.id == "virtual")
            .unwrap()
            .folded
    );
    restarted.set_protected("virtual", false).unwrap();
    restarted.set_protected("virtual", true).unwrap();
    let device = restarted
        .state(0)
        .devices
        .into_iter()
        .find(|d| d.id == "virtual")
        .unwrap();
    assert!(device.folded && device.protected);
}
#[test]
fn pause_expires_at_one_hour_and_manual_resume_is_immediate() {
    for manual in [false, true] {
        let mut audio = SimulatedAudio::speaker(100.0);
        let mut guard = Guard::new(Settings::default());
        guard.refresh(&mut audio).unwrap();
        guard.pause(20);
        guard.enforce(&mut audio, 3619).unwrap();
        assert!(audio.writes.is_empty());
        assert_eq!(guard.state(3619).pause_remaining, 1);
        if manual {
            guard.resume();
        }
        guard
            .enforce(&mut audio, if manual { 100 } else { 3620 })
            .unwrap();
        assert_eq!(audio.writes, [("usb".into(), 10.0)]);
        assert_eq!(guard.state(3620).pause_remaining, 0);
    }
}
#[test]
fn changed_settings_apply_and_backend_failures_are_reported() {
    let mut audio = SimulatedAudio::speaker(80.0);
    let mut guard = Guard::new(Settings::default());
    guard.refresh(&mut audio).unwrap();
    assert!(guard.set_levels(20, 30).is_err());
    assert!(guard.set_protected("not-a-device", false).is_err());
    guard.set_levels(30, 5).unwrap();
    guard.enforce(&mut audio, 0).unwrap();
    assert_eq!(audio.writes, [("usb".into(), 30.0)]);
    audio.fail = true;
    assert_eq!(guard.enforce(&mut audio, 0).unwrap_err(), "Disconnected");
    guard.set_protected("usb", false).unwrap();
    audio.fail = false;
    guard.enforce(&mut audio, 0).unwrap();
    audio.current = None;
    guard.refresh(&mut audio).unwrap();
    guard.enforce(&mut audio, 0).unwrap();
}
#[test]
fn saved_pause_minutes_control_new_pause_but_keep_existing_deadline() {
    let mut audio = SimulatedAudio::speaker(100.0);
    let mut guard = Guard::new(Settings::default());
    guard.refresh(&mut audio).unwrap();
    for invalid in [0, 721] {
        assert!(guard.set_pause_minutes(invalid).is_err());
    }
    guard.set_pause_minutes(15).unwrap();
    guard.pause(0);
    guard.set_pause_minutes(120).unwrap();
    guard.enforce(&mut audio, 899).unwrap();
    assert!(audio.writes.is_empty());
    guard.enforce(&mut audio, 900).unwrap();
    assert_eq!(audio.writes, [("usb".into(), 10.0)]);
    guard.pause(1000);
    assert_eq!(guard.state(1000).pause_remaining, 7200);
}
