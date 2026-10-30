use crate::settings::Settings;
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub id: String,
    pub name: String,
    pub software: bool,
    pub display: bool,
    pub headphones: bool,
}
impl Endpoint {
    pub fn folded(&self) -> bool {
        self.software || self.display
    }
}
pub trait AudioBackend {
    fn active_devices(&mut self) -> Result<Vec<Endpoint>, String>;
    fn default_output(&mut self) -> Result<Option<String>, String>;
    fn volume(&mut self, id: &str) -> Result<f32, String>;
    fn set_volume(&mut self, id: &str, percent: f32) -> Result<(), String>;
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub protected: bool,
    pub folded: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub cap: u8,
    pub drop: u8,
    pub language_mode: String,
    pub pause_minutes: u16,
    pub current_volume: Option<u8>,
    pub devices: Vec<Device>,
    pub current: Option<String>,
    pub pause_remaining: u64,
    pub reduction: Option<u8>,
    pub error: Option<String>,
}
pub struct Guard {
    pub settings: Settings,
    devices: Vec<Endpoint>,
    current: Option<String>,
    pause_until: Option<u64>,
    reduction: Option<u8>,
    volume: Option<u8>,
}
impl Guard {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            devices: vec![],
            current: None,
            pause_until: None,
            reduction: None,
            volume: None,
        }
    }
    pub fn sync_default(&mut self, audio: &mut impl AudioBackend) -> Result<bool, String> {
        let current = audio.default_output()?;
        let before = self.settings.clone();
        if let Some(id) = &current {
            self.settings
                .devices
                .entry(id.clone())
                .or_default()
                .was_default = true;
        }
        if current != self.current {
            self.reduction = None;
            self.volume = None;
        }
        self.current = current;
        Ok(before != self.settings)
    }
    pub fn refresh(&mut self, audio: &mut impl AudioBackend) -> Result<bool, String> {
        let mut changed = self.sync_default(audio)?;
        self.devices = audio.active_devices()?;
        for device in &self.devices {
            if !self.settings.devices.contains_key(&device.id) {
                self.settings
                    .devices
                    .insert(device.id.clone(), Default::default());
                changed = true;
            }
        }
        Ok(changed)
    }
    pub fn enforce(&mut self, audio: &mut impl AudioBackend, now: u64) -> Result<(), String> {
        let Some(id) = &self.current else {
            return Ok(());
        };
        let volume = audio.volume(id)?;
        if !volume.is_finite() || !(0.0..=100.01).contains(&volume) {
            return Err("Invalid volume from audio backend".into());
        }
        self.volume = Some(volume.round().min(100.0) as u8);
        if self.pause_until.is_some_and(|until| now < until) {
            return Ok(());
        }
        self.pause_until = None;
        if !self.settings.devices.get(id).is_none_or(|s| s.protected) {
            return Ok(());
        }
        let target = if volume >= 99.5 {
            Some(self.settings.drop)
        } else if volume > self.settings.cap as f32 {
            Some(self.settings.cap)
        } else {
            None
        };
        if let Some(target) = target {
            if volume > target as f32 {
                audio.set_volume(id, target as f32)?;
                self.reduction = Some(target);
                self.volume = Some(target);
            }
        }
        Ok(())
    }
    pub fn set_pause_minutes(&mut self, minutes: u16) -> Result<(), String> {
        if !(1..=720).contains(&minutes) {
            return Err("Pause duration must be 1 to 720 whole minutes".into());
        }
        self.settings.pause_minutes = minutes;
        Ok(())
    }
    pub fn pause(&mut self, now: u64) {
        self.pause_until = Some(now + u64::from(self.settings.pause_minutes) * 60);
    }
    pub fn resume(&mut self) {
        self.pause_until = None;
    }
    pub fn set_protected(&mut self, id: &str, protected: bool) -> Result<(), String> {
        let setting = self
            .settings
            .devices
            .get_mut(id)
            .ok_or("Unknown output device")?;
        setting.protected = protected;
        self.reduction = None;
        Ok(())
    }
    pub fn set_levels(&mut self, cap: u8, drop: u8) -> Result<(), String> {
        if !(10..=100).contains(&cap) || drop > cap {
            return Err("Use 0 <= drop <= cap and 10 <= cap <= 100".into());
        }
        self.settings.cap = cap;
        self.settings.drop = drop;
        Ok(())
    }
    pub fn state(&self, now: u64) -> State {
        State {
            cap: self.settings.cap,
            drop: self.settings.drop,
            language_mode: self.settings.language.clone(),
            pause_minutes: self.settings.pause_minutes,
            current_volume: self.volume,
            devices: self
                .devices
                .iter()
                .map(|d| Device {
                    id: d.id.clone(),
                    name: d.name.clone(),
                    kind: if d.display {
                        "display"
                    } else if d.software {
                        "software"
                    } else if d.headphones {
                        "headphones"
                    } else {
                        "speaker"
                    }
                    .into(),
                    protected: self.settings.devices.get(&d.id).is_none_or(|s| s.protected),
                    folded: d.folded(),
                })
                .collect(),
            current: self.current.clone(),
            pause_remaining: self.pause_until.unwrap_or(0).saturating_sub(now),
            reduction: self.reduction,
            error: None,
        }
    }
}
