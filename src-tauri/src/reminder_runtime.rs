use crate::{
    SettingsLock, preferences,
    preferences::Preferences,
    reminders::{self, Gate},
};
use chrono::{DateTime, FixedOffset, Local, NaiveDate};
use serde::Serialize;
use std::{collections::BTreeSet, time::Instant};
use tauri::{Manager, UserAttentionType};
use tauri_plugin_opener::OpenerExt;

#[derive(Default)]
pub struct Session {
    pub gate: Option<Gate>,
    last_attention: Option<DateTime<FixedOffset>>,
    last_dates: BTreeSet<NaiveDate>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    preferences: Option<Preferences>,
    pending: BTreeSet<NaiveDate>,
    today: NaiveDate,
    snooze_until: Option<DateTime<FixedOffset>>,
    snoozed: bool,
    next_reminder: Option<String>,
    confirmation_seconds: Option<u64>,
}

enum Action {
    Read,
    Tick,
    Open { dates: BTreeSet<NaiveDate> },
    Complete { dates: BTreeSet<NaiveDate> },
    Snooze { minutes: u32 },
}

pub fn show(app: &tauri::AppHandle, settings: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.eval(if settings {
            "window.location.assign('/settings/')"
        } else {
            "window.location.assign('/')"
        }) {
            log::error!("Could not navigate BYT: {error}");
        }
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn apply(app: &tauri::AppHandle, action: Action) -> Result<Snapshot, String> {
    let shared = app.state::<SettingsLock>();
    let mut session = shared
        .0
        .lock()
        .map_err(|_| "Reminders are busy. Please restart BYT.")?;
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    let preferences = preferences::load(&directory.join("preferences.json"))?;
    let now = Local::now().fixed_offset();
    if preferences.is_none() {
        return match action {
            Action::Read | Action::Tick => Ok(Snapshot {
                preferences: None,
                pending: BTreeSet::new(),
                today: now.date_naive(),
                snooze_until: None,
                snoozed: false,
                next_reminder: None,
                confirmation_seconds: None,
            }),
            _ => Err("Set up your timesheet first.".into()),
        };
    }
    let path = directory.join("reminders.json");
    let mut state = reminders::load(&path, now)?;
    let before = state.clone();
    if let Some(preferences) = &preferences {
        state.advance(preferences, now);
    }
    let tick = matches!(action, Action::Tick);
    match action {
        Action::Open { dates } => {
            session.gate = None;
            if dates != state.pending {
                return Err(
                    "The pending dates have changed. Review them and open the timesheet again."
                        .into(),
                );
            }
            let preferences = preferences.as_ref().ok_or("Set up your timesheet first.")?;
            app.opener()
                .open_url(&preferences.url, None::<&str>)
                .map_err(|error| format!("Could not open your timesheet: {error}"))?;
            if !dates.is_empty() {
                session.gate = Some(Gate {
                    opened_at: Instant::now(),
                    dates,
                });
            }
        }
        Action::Complete { dates } => {
            let gate = session
                .gate
                .as_ref()
                .ok_or("Open the timesheet before confirming submission.")?;
            if gate.dates != dates || dates.is_empty() {
                return Err(
                    "The listed dates changed. Open the timesheet again before confirming.".into(),
                );
            }
            state.complete(gate, Instant::now())?;
        }
        Action::Snooze { minutes } => {
            state.snooze(now, minutes)?;
        }
        Action::Read | Action::Tick => {}
    }
    // Checkpoint the cursor at most once per minute; mutations are always durable
    // before their successful result is returned to the window.
    if state.pending != before.pending
        || state.completed != before.completed
        || state.snooze_until != before.snooze_until
        || (state.checked_at - before.checked_at).num_seconds() >= 60
        || !path.exists()
    {
        reminders::save(&path, &state)?;
    }
    if state.pending.is_empty() || state.snooze_until != before.snooze_until {
        session.gate = None;
        session.last_attention = None;
    }
    if tick && state.interrupt(now) {
        let interval = preferences
            .as_ref()
            .map_or(5, |value| value.repeat_interval);
        if session
            .last_attention
            .is_none_or(|last| (now - last).num_seconds() >= i64::from(interval) * 60)
            || state.pending != session.last_dates
        {
            session.last_attention = Some(now);
            session.last_dates = state.pending.clone();
            show(app, false);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.request_user_attention(Some(UserAttentionType::Informational));
            }
        }
    }
    let confirmation_seconds = session
        .gate
        .as_ref()
        .filter(|gate| gate.dates == state.pending)
        .map(|gate| 5_u64.saturating_sub(gate.opened_at.elapsed().as_secs()));
    Ok(Snapshot {
        next_reminder: preferences
            .as_ref()
            .and_then(|value| reminders::next_occurrence(value, now, &state.completed)),
        preferences,
        pending: state.pending,
        today: now.date_naive(),
        snooze_until: state.snooze_until,
        snoozed: state.snooze_until.is_some_and(|deadline| now < deadline),
        confirmation_seconds,
    })
}

async fn perform(app: tauri::AppHandle, action: Action) -> Result<Snapshot, String> {
    tauri::async_runtime::spawn_blocking(move || apply(&app, action))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn load_reminder(app: tauri::AppHandle) -> Result<Snapshot, String> {
    perform(app, Action::Read).await
}
#[tauri::command]
pub async fn open_timesheet(
    app: tauri::AppHandle,
    dates: BTreeSet<NaiveDate>,
) -> Result<Snapshot, String> {
    perform(app, Action::Open { dates }).await
}
#[tauri::command]
pub async fn complete_reminder(
    app: tauri::AppHandle,
    dates: BTreeSet<NaiveDate>,
) -> Result<Snapshot, String> {
    perform(app, Action::Complete { dates }).await
}
#[tauri::command]
pub async fn snooze_reminder(app: tauri::AppHandle, minutes: u32) -> Result<Snapshot, String> {
    perform(app, Action::Snooze { minutes }).await
}

pub fn start(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            if let Err(error) = apply(&app, Action::Tick) {
                log::error!("Reminder check failed: {error}");
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}
