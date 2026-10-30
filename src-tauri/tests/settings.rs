use earmuff::settings::{DeviceSetting, Settings};
#[test]
fn settings_round_trip_stable_ids_and_safe_defaults() {
    let mut settings = Settings {
        cap: 35,
        drop: 7,
        pause_minutes: 45,
        language: "zh".into(),
        ..Default::default()
    };
    settings.devices.insert(
        "{0.0.0.00000000}.{endpoint-中文}".into(),
        DeviceSetting {
            protected: false,
            was_default: true,
        },
    );
    assert_eq!(Settings::parse(&settings.encode()).unwrap(), settings);
    for malformed in [
        "[Settings]\nPauseMinutes=0",
        "[Settings]\nPauseMinutes=721",
        "[Settings]\nPauseMinutes=1.5",
        "[Settings]\nLanguage=ja",
        "[Settings]\nCap=garbage",
        "[Settings]\nCap=900",
        "[Settings]\nCap=5\nDrop=10",
        "[Device:zz]\nProtected=false",
        "garbled",
        "[Settings\nCap=30",
        "[Device:61]\nProtected=maybe",
    ] {
        assert_eq!(
            Settings::parse(malformed).unwrap_or_default(),
            Settings::default(),
            "{malformed}"
        );
    }
    assert_eq!(Settings::parse("").unwrap(), Settings::default());
}
