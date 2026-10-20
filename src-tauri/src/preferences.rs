use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub url: String,
    pub schedule: Schedule,
    pub time: String,
    pub repeat_interval: u32,
    pub start_at_login: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Schedule {
    Weekly { days: Vec<u8> },
    Monthly { day: MonthDay },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MonthDay {
    Number(u8),
    Last(LastDay),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LastDay {
    Last,
}

impl Preferences {
    pub fn validate(&self) -> Result<(), String> {
        let url = tauri::Url::parse(&self.url)
            .map_err(|_| "Enter a full HTTP or HTTPS timesheet URL.")?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !self
                .url
                .to_lowercase()
                .starts_with(&format!("{}://", url.scheme()))
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("Use an HTTP or HTTPS URL without a username or password.".into());
        }
        match &self.schedule {
            Schedule::Weekly { days } if days.is_empty() || days.iter().any(|day| *day > 6) => {
                return Err("Choose at least one valid weekday.".into());
            }
            Schedule::Monthly {
                day: MonthDay::Number(day),
            } if !(1..=31).contains(day) => {
                return Err("Choose a day from 1 to 31 or the last day of the month.".into());
            }
            _ => {}
        }
        let bytes = self.time.as_bytes();
        if bytes.len() != 5
            || bytes[2] != b':'
            || ![bytes[0], bytes[1], bytes[3], bytes[4]]
                .iter()
                .all(u8::is_ascii_digit)
            || &self.time[..2] > "23"
            || &self.time[3..] > "59"
        {
            return Err("Choose a valid reminder time.".into());
        }
        if ![1, 5, 10, 15, 30, 60].contains(&self.repeat_interval) {
            return Err("Choose a supported repeat interval.".into());
        }
        Ok(())
    }
}

pub fn load(path: &Path) -> Result<Option<Preferences>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Could not read saved settings: {error}")),
    };
    let preferences: Preferences = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Could not read saved settings: {error}"))?;
    preferences.validate()?;
    Ok(Some(preferences))
}

pub fn save(path: &Path, preferences: &Preferences) -> Result<(), String> {
    use std::io::Write;

    preferences.validate()?;
    let directory = path.parent().ok_or("Settings directory is unavailable.")?;
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("Could not create settings directory: {error}"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)
        .map_err(|error| format!("Could not prepare settings file: {error}"))?;
    serde_json::to_writer_pretty(&mut temporary, preferences)
        .map_err(|error| format!("Could not write settings: {error}"))?;
    temporary
        .flush()
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|error| format!("Could not save settings: {error}"))?;
    temporary
        .persist(path)
        .map_err(|error| format!("Could not replace settings: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weekly() -> Preferences {
        Preferences {
            url: "https://example.com/timesheet".into(),
            schedule: Schedule::Weekly { days: vec![0, 4] },
            time: "17:00".into(),
            repeat_interval: 5,
            start_at_login: true,
        }
    }

    #[test]
    fn setup_survives_reopening_and_settings_replace_saved_options() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        assert_eq!(load(&path).unwrap(), None);
        let initial = weekly();
        save(&path, &initial).unwrap();
        assert_eq!(load(&path).unwrap(), Some(initial.clone()));

        let updated = Preferences {
            url: "http://localhost:8080/hours".into(),
            schedule: Schedule::Monthly {
                day: MonthDay::Last(LastDay::Last),
            },
            time: "09:30".into(),
            repeat_interval: 15,
            start_at_login: false,
        };
        save(&path, &updated).unwrap();
        assert_eq!(load(&path).unwrap(), Some(updated));
        assert!(!directory.path().join("reminders.json").exists());
    }

    #[test]
    fn ordinary_monthly_days_round_trip_without_changing_reminder_state() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let reminders = directory.path().join("reminders.json");
        std::fs::write(
            &reminders,
            r#"{"pending":["2026-10-01"],"snoozeUntil":"2026-10-20T17:00:00Z"}"#,
        )
        .unwrap();
        let before = std::fs::read(&reminders).unwrap();
        for day in [1, 28, 29, 30, 31] {
            let preferences = Preferences {
                schedule: Schedule::Monthly {
                    day: MonthDay::Number(day),
                },
                ..weekly()
            };
            save(&path, &preferences).unwrap();
            assert_eq!(load(&path).unwrap(), Some(preferences));
        }
        assert_eq!(std::fs::read(reminders).unwrap(), before);
    }

    #[test]
    fn write_failure_is_reported() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        std::fs::create_dir(&path).unwrap();
        assert!(save(&path, &weekly()).is_err());
        assert!(path.is_dir());
    }

    #[test]
    fn invalid_edits_do_not_replace_saved_settings() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let initial = weekly();
        save(&path, &initial).unwrap();
        let invalid = [
            Preferences {
                url: "javascript:alert(1)".into(),
                ..initial.clone()
            },
            Preferences {
                url: "https://user:secret@example.com".into(),
                ..initial.clone()
            },
            Preferences {
                schedule: Schedule::Weekly { days: vec![] },
                ..initial.clone()
            },
            Preferences {
                schedule: Schedule::Weekly { days: vec![7] },
                ..initial.clone()
            },
            Preferences {
                schedule: Schedule::Monthly {
                    day: MonthDay::Number(0),
                },
                ..initial.clone()
            },
            Preferences {
                schedule: Schedule::Monthly {
                    day: MonthDay::Number(32),
                },
                ..initial.clone()
            },
            Preferences {
                time: "25:00".into(),
                ..initial.clone()
            },
            Preferences {
                repeat_interval: 0,
                ..initial.clone()
            },
        ];
        for preferences in invalid {
            assert!(
                save(&path, &preferences).is_err(),
                "accepted invalid preferences: {preferences:?}"
            );
            assert_eq!(load(&path).unwrap(), Some(initial.clone()));
        }
    }

    #[test]
    fn corrupt_storage_is_reported_instead_of_resetting_setup() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        std::fs::write(&path, "broken json").unwrap();
        assert!(load(&path).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "broken json");
    }
}
