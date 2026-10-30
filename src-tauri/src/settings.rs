use std::{collections::BTreeMap, fs, io::Write, path::Path};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceSetting {
    pub protected: bool,
    pub was_default: bool,
}
impl Default for DeviceSetting {
    fn default() -> Self {
        Self {
            protected: true,
            was_default: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub cap: u8,
    pub drop: u8,
    pub language: String,
    pub pause_minutes: u16,
    pub devices: BTreeMap<String, DeviceSetting>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            cap: 42,
            drop: 10,
            language: "system".into(),
            pause_minutes: 60,
            devices: BTreeMap::new(),
        }
    }
}
impl Settings {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut settings = Self::default();
        let mut section = String::new();
        let mut device_id = None;
        for line in text.trim_start_matches('\u{feff}').lines().map(str::trim) {
            if line.is_empty() || line.starts_with([';', '#']) {
                continue;
            }
            if line.starts_with('[') {
                section = line
                    .strip_prefix('[')
                    .and_then(|s| s.strip_suffix(']'))
                    .ok_or("Malformed INI section")?
                    .into();
                device_id = if let Some(hex) = section.strip_prefix("Device:") {
                    if hex.is_empty() || hex.len() % 2 != 0 || !hex.is_ascii() {
                        return Err("Invalid endpoint ID".into());
                    }
                    let bytes = (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| "Invalid endpoint ID")?;
                    let id = String::from_utf8(bytes).map_err(|_| "Invalid endpoint ID")?;
                    settings.devices.entry(id.clone()).or_default();
                    Some(id)
                } else {
                    None
                };
                continue;
            }
            let (key, value) = line.split_once('=').ok_or("Malformed INI setting")?;
            let (key, value) = (key.trim(), value.trim());
            if section == "Settings" {
                match key {
                    "Language" => {
                        if !["system", "en", "zh"].contains(&value) {
                            return Err("Invalid language".into());
                        }
                        settings.language = value.into();
                    }
                    "PauseMinutes" => {
                        settings.pause_minutes =
                            value.parse().map_err(|_| "Invalid pause duration")?
                    }
                    "Cap" => settings.cap = value.parse().map_err(|_| "Invalid cap")?,
                    "Drop" => settings.drop = value.parse().map_err(|_| "Invalid drop")?,
                    _ => {}
                }
            } else if let Some(id) = &device_id {
                let setting = settings
                    .devices
                    .get_mut(id)
                    .expect("Device inserted at section header");
                match key {
                    "Protected" => {
                        setting.protected = value.parse().map_err(|_| "Invalid protected switch")?
                    }
                    "WasDefault" => {
                        setting.was_default =
                            value.parse().map_err(|_| "Invalid default history")?
                    }
                    _ => {}
                }
            }
        }
        if !(1..=720).contains(&settings.pause_minutes) {
            return Err("Pause duration must be 1 to 720 whole minutes".into());
        }
        if !(10..=100).contains(&settings.cap) || settings.drop > settings.cap {
            return Err("Use 0 <= drop <= cap and 10 <= cap <= 100".into());
        }
        Ok(settings)
    }
    pub fn encode(&self) -> String {
        let mut text = format!(
            "[Settings]\nCap={}\nDrop={}\nLanguage={}\nPauseMinutes={}\n",
            self.cap, self.drop, self.language, self.pause_minutes
        );
        for (id, setting) in &self.devices {
            let hex: String = id.as_bytes().iter().map(|v| format!("{v:02x}")).collect();
            text.push_str(&format!(
                "\n[Device:{hex}]\nProtected={}\nWasDefault={}\n",
                setting.protected, setting.was_default
            ));
        }
        text
    }
    pub fn load(path: &Path) -> (Self, Option<String>) {
        match fs::read_to_string(path) {
            Ok(text) => match Self::parse(&text) {
                Ok(settings) => (settings, None),
                Err(error) => (
                    Self::default(),
                    Some(format!(
                        "Settings could not be read; safe defaults are active. {error}"
                    )),
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Self::default(), None),
            Err(error) => (
                Self::default(),
                Some(format!(
                    "Settings could not be read; safe defaults are active. {error}"
                )),
            ),
        }
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let parent = path.parent().ok_or("Settings directory is missing")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temporary = path.with_extension("ini.tmp");
        let result = (|| -> std::io::Result<()> {
            let mut file = fs::File::create(&temporary)?;
            file.write_all(self.encode().as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result.map_err(|e| format!("Settings could not be saved: {e}"))
    }
}
