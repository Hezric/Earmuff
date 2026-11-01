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
        "pause_minutes=0",
        "pause_minutes=721",
        "pause_minutes=1.5",
        "language='ja'",
        "cap='garbage'",
        "cap=900",
        "cap=5\ndrop=10",
        "[devices.'endpoint']\nprotected='maybe'",
        "cap=35\ncap=42",
        "unknown=true",
        "[devices.\"\"]\nprotected=false",
        "garbled",
    ] {
        assert!(Settings::parse(malformed).is_err(), "{malformed}");
    }
    assert_eq!(Settings::parse("").unwrap(), Settings::default());
}

#[test]
fn toml_settings_preserve_device_rules_without_ini_compatibility() {
    let settings = Settings::parse(
        r#"
cap = 35
drop = 7
language = "zh"
pause_minutes = 45

[devices."{endpoint-中文}"]
protected = false
was_default = true
"#,
    )
    .unwrap();
    assert_eq!(
        (settings.cap, settings.drop, settings.pause_minutes),
        (35, 7, 45)
    );
    assert_eq!(settings.language, "zh");
    assert_eq!(
        settings.devices["{endpoint-中文}"],
        DeviceSetting {
            protected: false,
            was_default: true
        }
    );
    assert_eq!(Settings::parse(&settings.encode()).unwrap(), settings);
    assert!(Settings::parse("[Settings]\nCap=35\nDrop=7").is_err());
}

#[test]
fn invalid_toml_preserves_the_file_and_repeated_saves_stay_valid() {
    let directory = std::env::temp_dir().join(format!("earmuff-toml-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("Earmuff.toml");
    let result = std::panic::catch_unwind(|| {
        assert_eq!(Settings::load(&path), (Settings::default(), None));
        std::fs::write(&path, "cap = 'broken'").unwrap();
        let (settings, error) = Settings::load(&path);
        assert_eq!(settings, Settings::default());
        assert!(error.is_some());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "cap = 'broken'");
        settings.save(&path).unwrap();
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), (settings.clone(), None));
        let bytes = std::fs::read(&path).unwrap();
        let invalid = Settings { cap: 5, ..settings };
        assert!(invalid.save(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(!path.with_extension("toml.tmp").exists());
    });
    std::fs::remove_dir_all(directory).unwrap();
    result.unwrap();
}
