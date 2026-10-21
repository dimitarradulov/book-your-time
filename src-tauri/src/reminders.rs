use crate::preferences::{MonthDay, Preferences, Schedule};
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::{Duration as Elapsed, Instant};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReminderState {
    pub checked_at: DateTime<FixedOffset>,
    pub pending: BTreeSet<NaiveDate>,
    pub completed: BTreeSet<NaiveDate>,
    pub snooze_until: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub prompted_months: BTreeSet<NaiveDate>,
}

impl ReminderState {
    pub fn new(now: DateTime<FixedOffset>) -> Self {
        Self {
            checked_at: now
                - Duration::seconds(i64::from(now.time().num_seconds_from_midnight()) + 1),
            pending: BTreeSet::new(),
            completed: BTreeSet::new(),
            snooze_until: None,
            prompted_months: BTreeSet::new(),
        }
    }
    pub fn advance(&mut self, preferences: &Preferences, now: DateTime<FixedOffset>) {
        let cursor = self.checked_at.naive_local();
        let timezone_changed = self.checked_at.offset() != now.offset();
        let time = NaiveTime::parse_from_str(&preferences.time, "%H:%M").expect("validated time");
        let mut date = cursor.date().min(now.date_naive());
        while date <= now.date_naive() {
            let due = date.and_time(time);
            if matches_date(preferences, date)
                && (due > cursor
                    || ((timezone_changed
                        || matches!(preferences.schedule, Schedule::Monthly { .. }))
                        && date == now.date_naive()))
                && due <= now.naive_local()
                && !self.completed.contains(&date)
            {
                self.pending.insert(date);
            }
            let Some(next) = date.succ_opt() else { break };
            date = next;
        }
        if now > self.checked_at {
            self.checked_at = now;
        }
    }
    pub fn interrupt(&self, now: DateTime<FixedOffset>) -> bool {
        self.pending.contains(&now.date_naive())
            && self.snooze_until.is_none_or(|deadline| now >= deadline)
    }
    pub fn snooze(&mut self, now: DateTime<FixedOffset>, minutes: u32) -> Result<(), String> {
        if ![15, 30, 60].contains(&minutes) {
            return Err("Choose 15 minutes, 30 minutes, or one hour.".into());
        }
        if self.pending.is_empty() {
            return Err("There are no pending reminders to snooze.".into());
        }
        self.snooze_until = Some(now + Duration::minutes(i64::from(minutes)));
        Ok(())
    }
    pub fn complete(&mut self, gate: &Gate, now: Instant) -> Result<(), String> {
        if !gate.ready(now) {
            return Err("Wait five seconds after opening the timesheet.".into());
        }
        for date in &gate.dates {
            if self.pending.remove(date) {
                self.completed.insert(*date);
            }
        }
        self.snooze_until = None;
        Ok(())
    }
}

fn matches_date(preferences: &Preferences, date: NaiveDate) -> bool {
    match &preferences.schedule {
        Schedule::Monthly { dates } => dates.contains(&date),
        Schedule::Weekly { days } => days.contains(&(date.weekday().num_days_from_monday() as u8)),
        Schedule::LegacyMonthly {
            day: MonthDay::Number(day),
        } => date.day() == u32::from(*day),
        Schedule::LegacyMonthly {
            day: MonthDay::Last(_),
        } => date
            .succ_opt()
            .is_some_and(|next| next.month() != date.month()),
    }
}

pub fn next_occurrence(
    preferences: &Preferences,
    now: DateTime<FixedOffset>,
    completed: &BTreeSet<NaiveDate>,
) -> Option<String> {
    let time = NaiveTime::parse_from_str(&preferences.time, "%H:%M").ok()?;
    if let Schedule::Monthly { dates } = &preferences.schedule {
        return dates
            .iter()
            .filter(|date| !completed.contains(date) && date.and_time(time) > now.naive_local())
            .min()
            .map(|date| date.and_time(time).format("%Y-%m-%dT%H:%M:%S").to_string());
    }
    let mut date = now.date_naive();
    // A valid monthly rule has an occurrence within the next two months.
    for _ in 0..=62 {
        if matches_date(preferences, date)
            && !completed.contains(&date)
            && date.and_time(time) > now.naive_local()
        {
            // Keep this a local date-time: the frontend applies the computer's timezone
            // at that future date, including its daylight-saving offset.
            return Some(date.and_time(time).format("%Y-%m-%dT%H:%M:%S").to_string());
        }
        date = date.succ_opt()?;
    }
    None
}

pub fn missing_months(preferences: &Preferences, today: NaiveDate) -> BTreeSet<NaiveDate> {
    let Schedule::Monthly { dates } = &preferences.schedule else {
        return BTreeSet::new();
    };
    let current = today.with_day(1).expect("first day");
    let next = if current.month() == 12 {
        NaiveDate::from_ymd_opt(current.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1)
    };
    std::iter::once(current)
        .chain(next.filter(|next| (*next - today).num_days() <= 7))
        .filter(|month| {
            !dates
                .iter()
                .any(|date| date.year() == month.year() && date.month() == month.month())
        })
        .collect()
}

impl ReminderState {
    pub fn take_missing_prompts(&mut self, preferences: &Preferences, today: NaiveDate) -> bool {
        let missing = missing_months(preferences, today);
        let prompt = missing
            .iter()
            .any(|month| !self.prompted_months.contains(month));
        self.prompted_months.extend(missing);
        prompt
    }
}

pub fn load(path: &std::path::Path, now: DateTime<FixedOffset>) -> Result<ReminderState, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ReminderState::new(now));
        }
        Err(error) => return Err(format!("Could not read reminders: {error}")),
    };
    let state: ReminderState = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Could not read reminders: {error}"))?;
    if !state.pending.is_disjoint(&state.completed) {
        return Err("Saved reminder dates conflict. Please restore your reminder file.".into());
    }
    Ok(state)
}

pub fn save(path: &std::path::Path, state: &ReminderState) -> Result<(), String> {
    use std::io::Write;
    let directory = path.parent().ok_or("Reminder directory unavailable.")?;
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(directory).map_err(|error| error.to_string())?;
    serde_json::to_writer_pretty(&mut temporary, state).map_err(|error| error.to_string())?;
    temporary
        .flush()
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    temporary
        .persist(path)
        .map_err(|error| format!("Could not save reminders: {error}"))?;
    Ok(())
}

#[derive(Clone)]
pub struct Gate {
    pub opened_at: Instant,
    pub dates: BTreeSet<NaiveDate>,
}
impl Gate {
    pub fn ready(&self, now: Instant) -> bool {
        now.duration_since(self.opened_at) >= Elapsed::from_secs(5)
    }
}
use chrono::Timelike;

#[cfg(test)]
mod tests {
    use super::*;
    fn at(value: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(value).unwrap()
    }
    fn weekly() -> Preferences {
        Preferences {
            url: "https://example.com".into(),
            schedule: Schedule::Weekly { days: vec![4] },
            time: "17:00".into(),
            repeat_interval: 5,
            start_at_login: false,
            reminder_sound: true,
        }
    }
    #[test]
    fn missed_dates_carry_forward_without_interrupting_unrelated_days() {
        let mut state = ReminderState::new(at("2026-10-16T16:00:00+03:00"));
        state.advance(&weekly(), at("2026-10-20T12:00:00+03:00"));
        assert_eq!(
            state
                .pending
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["2026-10-16"]
        );
        assert!(!state.interrupt(at("2026-10-20T12:00:00+03:00")));
        state.advance(&weekly(), at("2026-10-23T17:00:00+03:00"));
        assert_eq!(state.pending.len(), 2);
        assert!(state.interrupt(at("2026-10-23T17:00:00+03:00")));
    }
    #[test]
    fn snooze_deadline_survives_restart_and_never_completes_dates() {
        let now = at("2026-10-23T17:00:00+03:00");
        for minutes in [15, 30, 60] {
            let mut state = ReminderState::new(now);
            state.advance(&weekly(), now);
            state.snooze(now, minutes).unwrap();
            let restored: ReminderState =
                serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
            assert_eq!(restored.pending.len(), 1);
            assert!(
                !restored
                    .interrupt(now + Duration::minutes(i64::from(minutes)) - Duration::seconds(1))
            );
            assert!(restored.interrupt(now + Duration::minutes(i64::from(minutes))));
        }
    }
    #[test]
    fn completion_requires_successful_handoff_and_five_seconds_and_clears_only_shown_dates() {
        let now = at("2026-10-23T17:00:00+03:00");
        let mut state = ReminderState::new(now);
        state.advance(&weekly(), now);
        let instant = Instant::now();
        let gate = Gate {
            opened_at: instant,
            dates: state.pending.clone(),
        };
        assert!(
            state
                .complete(&gate, instant + Elapsed::from_secs(4))
                .is_err()
        );
        state
            .complete(&gate, instant + Elapsed::from_secs(5))
            .unwrap();
        assert!(state.pending.is_empty());
        assert_eq!(state.completed.len(), 1);
        state.advance(&weekly(), now + Duration::minutes(10));
        assert!(state.pending.is_empty());
        state.advance(&weekly(), at("2026-10-30T17:00:00+02:00"));
        assert_eq!(state.pending.len(), 1);
        state
            .complete(&gate, instant + Elapsed::from_secs(6))
            .unwrap();
        assert_eq!(state.pending.len(), 1);
    }
    #[test]
    fn monthly_last_day_handles_leap_year_and_numbered_days_skip_short_months() {
        let mut preferences = weekly();
        preferences.schedule = Schedule::LegacyMonthly {
            day: MonthDay::Last(crate::preferences::LastDay::Last),
        };
        let mut state = ReminderState::new(at("2028-02-01T00:00:00+02:00"));
        state.advance(&preferences, at("2028-03-01T12:00:00+02:00"));
        assert_eq!(
            state
                .pending
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["2028-02-29"]
        );
        preferences.schedule = Schedule::LegacyMonthly {
            day: MonthDay::Number(31),
        };
        let mut state = ReminderState::new(at("2026-02-01T00:00:00+02:00"));
        state.advance(&preferences, at("2026-03-31T17:00:00+03:00"));
        assert_eq!(
            state
                .pending
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["2026-03-31"]
        );
    }
    #[test]
    fn pending_snooze_and_completion_round_trip_through_atomic_storage() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("reminders.json");
        let now = at("2026-10-23T17:00:00+03:00");
        let mut state = load(&path, at("2026-10-16T16:00:00+03:00")).unwrap();
        state.advance(&weekly(), now);
        state.snooze(now, 30).unwrap();
        save(&path, &state).unwrap();
        let mut restored = load(&path, now).unwrap();
        assert_eq!(restored, state);
        let instant = Instant::now();
        let gate = Gate {
            opened_at: instant,
            dates: restored.pending.clone(),
        };
        restored
            .complete(&gate, instant + Elapsed::from_secs(5))
            .unwrap();
        save(&path, &restored).unwrap();
        let mut reopened = load(&path, now).unwrap();
        reopened.advance(&weekly(), now + Duration::minutes(10));
        assert!(reopened.pending.is_empty());
        assert_eq!(reopened.completed.len(), 2);
        std::fs::write(&path, "broken json").unwrap();
        assert!(load(&path, now).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "broken json");
    }

    #[test]
    fn schedule_edits_preserve_old_dates_and_local_time_follows_travel() {
        let mut state = ReminderState::new(at("2026-10-16T16:00:00+03:00"));
        let now = at("2026-10-20T12:00:00+03:00");
        state.advance(&weekly(), now);
        let mut changed = weekly();
        changed.schedule = Schedule::Weekly { days: vec![1] };
        state.checked_at = now;
        state.advance(&changed, at("2026-10-20T16:59:00-04:00"));
        assert_eq!(state.pending.len(), 1);
        state.advance(&changed, at("2026-10-20T17:00:00-04:00"));
        assert_eq!(state.pending.len(), 2);
        assert_eq!(
            next_occurrence(&changed, at("2026-10-20T17:00:00-04:00"), &state.completed),
            Some("2026-10-27T17:00:00".into())
        );
    }
    #[test]
    fn travelling_past_local_due_time_catches_up_on_the_current_scheduled_day() {
        let mut state = ReminderState::new(at("2026-10-23T16:30:00-04:00"));
        state.advance(&weekly(), at("2026-10-23T16:30:00-04:00"));
        state.advance(&weekly(), at("2026-10-24T04:00:00+03:00"));
        // Arrival falls on Saturday: Friday should be retained for next time.
        assert_eq!(state.pending.len(), 1);
        let mut state = ReminderState::new(at("2026-10-23T09:30:00-04:00"));
        state.advance(&weekly(), at("2026-10-23T09:30:00-04:00"));
        state.advance(&weekly(), at("2026-10-23T17:01:00+03:00"));
        assert_eq!(state.pending.len(), 1);
        assert!(state.interrupt(at("2026-10-23T17:01:00+03:00")));
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
    fn arbitrary_monthly_dates_handle_leap_year_rollover_and_sparse_schedules() {
        let preferences = monthly(&["2027-12-29", "2028-02-29", "2028-10-23"]);
        let mut state = ReminderState::new(at("2027-12-01T12:00:00+02:00"));
        state.advance(&preferences, at("2028-03-01T12:00:00+02:00"));
        assert_eq!(
            state
                .pending
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["2027-12-29", "2028-02-29"]
        );
        assert!(!state.interrupt(at("2028-03-01T12:00:00+02:00")));
        assert_eq!(
            next_occurrence(
                &preferences,
                at("2028-03-01T12:00:00+02:00"),
                &state.completed
            ),
            Some("2028-10-23T17:00:00".into())
        );
        state.advance(&preferences, at("2028-10-23T17:00:00+03:00"));
        assert_eq!(state.pending.len(), 3);
        assert!(state.interrupt(at("2028-10-23T17:00:00+03:00")));
        assert_eq!(
            next_occurrence(
                &preferences,
                at("2028-10-23T17:00:00+03:00"),
                &state.completed
            ),
            None
        );
    }
    #[test]
    fn adding_today_after_shared_time_is_due_even_after_cursor_was_checked() {
        let now = at("2026-10-21T18:00:00+03:00");
        let mut state = ReminderState::new(now);
        state.advance(&monthly(&[]), now);
        state.advance(&monthly(&["2026-10-21"]), now);
        assert!(state.interrupt(now));
        state.completed.insert(now.date_naive());
        state.pending.clear();
        state.advance(&monthly(&["2026-10-21"]), now);
        assert!(state.pending.is_empty());
    }
    #[test]
    fn missing_month_prompt_is_seven_days_before_and_once_across_restart() {
        let preferences = monthly(&["2026-10-25"]);
        let mut state = ReminderState::new(at("2026-10-24T12:00:00+03:00"));
        assert!(!state.take_missing_prompts(&preferences, "2026-10-24".parse().unwrap()));
        assert!(state.take_missing_prompts(&preferences, "2026-10-25".parse().unwrap()));
        assert_eq!(
            missing_months(&preferences, "2026-10-25".parse().unwrap()),
            BTreeSet::from(["2026-11-01".parse().unwrap()])
        );
        let mut restored: ReminderState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert!(!restored.take_missing_prompts(&preferences, "2026-11-01".parse().unwrap()));
        assert_eq!(
            missing_months(&monthly(&["2026-11-25"]), "2026-11-01".parse().unwrap()),
            BTreeSet::new()
        );
        assert!(missing_months(&weekly(), "2026-10-25".parse().unwrap()).is_empty());
    }
    #[test]
    fn startup_catches_missing_current_month_and_old_state_defaults_prompt_markers() {
        let now = at("2027-01-02T12:00:00+02:00");
        let state = ReminderState::new(now);
        let mut json = serde_json::to_value(state).unwrap();
        json.as_object_mut().unwrap().remove("promptedMonths");
        let mut restored: ReminderState = serde_json::from_value(json).unwrap();
        assert!(restored.take_missing_prompts(&monthly(&[]), now.date_naive()));
        assert!(!restored.take_missing_prompts(&monthly(&[]), now.date_naive()));
        assert_eq!(
            missing_months(&monthly(&["2026-12-28"]), "2026-12-25".parse().unwrap()),
            BTreeSet::from(["2027-01-01".parse().unwrap()])
        );
    }
    #[test]
    fn converting_legacy_rule_preserves_missed_dates_and_snooze() {
        let old = Preferences {
            schedule: Schedule::LegacyMonthly {
                day: MonthDay::Number(25),
            },
            ..weekly()
        };
        let mut state = ReminderState::new(at("2026-09-24T12:00:00+03:00"));
        let now = at("2026-10-26T12:00:00+02:00");
        state.advance(&old, now);
        state.snooze(now, 30).unwrap();
        let snooze = state.snooze_until;
        state.advance(&monthly(&["2026-11-27"]), now);
        assert_eq!(state.pending.len(), 2);
        assert_eq!(state.snooze_until, snooze);
    }
}
