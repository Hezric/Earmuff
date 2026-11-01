use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::Write, path::Path};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
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
        let settings: Self = toml::from_str(text.trim_start_matches('\u{feff}'))
            .map_err(|e| format!("Invalid TOML settings: {e}"))?;
        if settings.devices.keys().any(|id| id.is_empty()) {
            return Err("Invalid endpoint ID".into());
        }
        if !["system", "en", "zh"].contains(&settings.language.as_str()) {
            return Err("Invalid language".into());
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
        toml::to_string_pretty(self).expect("settings contain only TOML-compatible types")
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
        Self::parse(&self.encode())?;
        let parent = path.parent().ok_or("Settings directory is missing")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temporary = path.with_extension("toml.tmp");
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
