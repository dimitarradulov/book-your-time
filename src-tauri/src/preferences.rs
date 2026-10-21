use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub url: String,
    pub schedule: Schedule,
    pub time: String,
    pub repeat_interval: u32,
    pub start_at_login: bool,
    #[serde(default = "sound_enabled_by_default")]
    pub reminder_sound: bool,
}

fn sound_enabled_by_default() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Schedule {
    Weekly { days: Vec<u8> },
    Monthly { dates: Vec<NaiveDate> },
    LegacyMonthly { day: MonthDay },
}

impl<'de> Deserialize<'de> for Schedule {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
        enum Stored {
            Weekly {
                days: Vec<u8>,
            },
            Monthly {
                dates: Option<Vec<NaiveDate>>,
                day: Option<MonthDay>,
            },
            LegacyMonthly {
                day: MonthDay,
            },
        }
        match Stored::deserialize(deserializer)? {
            Stored::Weekly { days } => Ok(Self::Weekly { days }),
            Stored::Monthly {
                dates: Some(dates),
                day: None,
            } => Ok(Self::Monthly { dates }),
            Stored::Monthly {
                dates: None,
                day: Some(day),
            }
            | Stored::LegacyMonthly { day } => Ok(Self::LegacyMonthly { day }),
            _ => Err(serde::de::Error::custom("Choose monthly booking dates.")),
        }
    }
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
            Schedule::LegacyMonthly {
                day: MonthDay::Number(day),
            } if !(1..=31).contains(day) => {
                return Err("Choose a day from 1 to 31 or the last day of the month.".into());
            }
            Schedule::Monthly { dates } => {
                let mut months = std::collections::BTreeSet::new();
                if dates
                    .iter()
                    .any(|date| !months.insert((date.year(), date.month())))
                {
                    return Err("Choose at most one booking date per month.".into());
                }
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
    pub fn validate_update(
        &self,
        previous: Option<&Self>,
        now: DateTime<FixedOffset>,
        occurred_dates: &BTreeSet<NaiveDate>,
    ) -> Result<(), String> {
        self.validate()?;
        let old_dates = match previous.map(|value| &value.schedule) {
            Some(Schedule::Monthly { dates }) => dates.as_slice(),
            _ => &[],
        };
        let dates = match &self.schedule {
            Schedule::Monthly { dates } => dates.as_slice(),
            Schedule::Weekly { .. } => &[],
            Schedule::LegacyMonthly { .. } => {
                return Err("Choose individual monthly booking dates before saving.".into());
            }
        };
        if dates
            .iter()
            .any(|date| *date < now.date_naive() && !old_dates.contains(date))
        {
            return Err("New booking dates must be today or later.".into());
        }
        if let Some(previous) = previous {
            let old_time =
                NaiveTime::parse_from_str(&previous.time, "%H:%M").expect("validated time");
            if matches!(self.schedule, Schedule::Monthly { .. })
                && old_dates.iter().any(|date| {
                    (date.and_time(old_time) <= now.naive_local() || occurred_dates.contains(date))
                        && !dates.contains(date)
                })
            {
                return Err("A booking date cannot change after its reminder becomes due.".into());
            }
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
            reminder_sound: true,
        }
    }

    #[test]
    fn existing_settings_enable_sound_without_changing_other_preferences() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let expected = weekly();
        let mut legacy = serde_json::to_value(&expected).unwrap();
        legacy.as_object_mut().unwrap().remove("reminderSound");
        std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();

        assert_eq!(load(&path).unwrap(), Some(expected));
    }

    #[test]
    fn disabling_and_reenabling_sound_survives_reopening() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let mut preferences = weekly();
        preferences.reminder_sound = false;
        save(&path, &preferences).unwrap();
        let mut reopened = load(&path).unwrap().unwrap();
        assert!(!reopened.reminder_sound);
        reopened.reminder_sound = true;
        save(&path, &reopened).unwrap();
        assert!(load(&path).unwrap().unwrap().reminder_sound);
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
                dates: vec![NaiveDate::from_ymd_opt(2028, 2, 29).unwrap()],
            },
            time: "09:30".into(),
            repeat_interval: 15,
            start_at_login: false,
            reminder_sound: false,
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
                    dates: vec![NaiveDate::from_ymd_opt(2026, 10, day as u32).unwrap()],
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
                schedule: Schedule::LegacyMonthly {
                    day: MonthDay::Number(0),
                },
                ..initial.clone()
            },
            Preferences {
                schedule: Schedule::LegacyMonthly {
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
    fn monthly(dates: &[&str]) -> Preferences {
        Preferences {
            schedule: Schedule::Monthly {
                dates: dates.iter().map(|date| date.parse().unwrap()).collect(),
            },
            ..weekly()
        }
    }
    #[test]
    fn monthly_dates_reject_duplicate_months_and_invalid_calendar_dates() {
        assert!(monthly(&["2026-10-20", "2026-10-21"]).validate().is_err());
        assert!(monthly(&["2026-10-20", "2026-10-20"]).validate().is_err());
        assert!(monthly(&[]).validate().is_ok());
        assert!(monthly(&["2028-02-29", "2029-02-28"]).validate().is_ok());
        let mut json = serde_json::to_value(monthly(&[])).unwrap();
        json["schedule"]["dates"] = serde_json::json!(["2027-02-29"]);
        assert!(serde_json::from_value::<Preferences>(json).is_err());
    }
    #[test]
    fn edits_allow_future_changes_and_preserve_due_dates_and_unchanged_past_dates() {
        let now = DateTime::parse_from_rfc3339("2026-10-21T18:00:00+03:00").unwrap();
        let old = monthly(&["2026-09-25", "2026-10-21", "2026-11-25"]);
        assert!(
            monthly(&["2026-09-25", "2026-10-21", "2026-11-27"])
                .validate_update(Some(&old), now, &BTreeSet::new())
                .is_ok()
        );
        assert!(
            monthly(&["2026-09-25", "2026-10-22", "2026-11-25"])
                .validate_update(Some(&old), now, &BTreeSet::new())
                .is_err()
        );
        assert!(
            monthly(&["2026-10-20"])
                .validate_update(None, now, &BTreeSet::new())
                .is_err()
        );
        assert!(
            monthly(&["2026-10-21"])
                .validate_update(None, now, &BTreeSet::new())
                .is_ok()
        );
        assert!(
            weekly()
                .validate_update(Some(&old), now, &BTreeSet::new())
                .is_ok()
        );
    }
    #[test]
    fn legacy_monthly_preferences_load_without_rewriting_and_convert_only_on_save() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("preferences.json");
        let mut json = serde_json::to_value(weekly()).unwrap();
        json["schedule"] = serde_json::json!({"kind": "monthly", "day": "last"});
        let bytes = serde_json::to_vec(&json).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let loaded = load(&path).unwrap().unwrap();
        assert!(matches!(loaded.schedule, Schedule::LegacyMonthly { .. }));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(
            loaded
                .validate_update(
                    Some(&loaded),
                    DateTime::parse_from_rfc3339("2026-10-21T18:00:00+03:00").unwrap(),
                    &BTreeSet::new(),
                )
                .is_err()
        );
        let converted = monthly(&["2026-10-31"]);
        save(&path, &converted).unwrap();
        assert_eq!(load(&path).unwrap(), Some(converted));
    }
    #[test]
    fn moving_shared_time_does_not_unlock_an_existing_occurrence() {
        let now = DateTime::parse_from_rfc3339("2026-10-21T18:00:00+03:00").unwrap();
        let original = monthly(&["2026-10-21", "2026-11-25"]);
        let occurred = BTreeSet::from(["2026-10-21".parse().unwrap()]);
        let later_time = Preferences {
            time: "19:00".into(),
            ..original.clone()
        };
        assert!(
            later_time
                .validate_update(Some(&original), now, &occurred)
                .is_ok()
        );
        let moved = Preferences {
            time: "19:00".into(),
            ..monthly(&["2026-10-22", "2026-11-25"])
        };
        assert!(
            moved
                .validate_update(Some(&later_time), now, &occurred)
                .is_err()
        );
        assert!(
            monthly(&["2026-10-21", "2026-11-26"])
                .validate_update(Some(&later_time), now, &occurred)
                .is_ok()
        );
    }
}
