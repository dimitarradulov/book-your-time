mod preferences;
mod reminder_runtime;
mod reminders;

use preferences::Preferences;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

#[derive(Clone, Default)]
struct SettingsLock(Arc<Mutex<reminder_runtime::Session>>);

#[tauri::command]
async fn load_preferences(app: tauri::AppHandle) -> Result<Option<Preferences>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = app
            .path()
            .app_config_dir()
            .map_err(|error| error.to_string())?;
        preferences::load(&path.join("preferences.json"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn save_preferences(
    app: tauri::AppHandle,
    state: tauri::State<'_, SettingsLock>,
    mut preferences: Preferences,
) -> Result<Preferences, String> {
    let lock = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut session = lock
            .0
            .lock()
            .map_err(|_| "Settings are busy. Please restart BYT.")?;
        preferences.url = preferences.url.trim().to_string();
        preferences.validate()?;
        let path = app
            .path()
            .app_config_dir()
            .map_err(|error| error.to_string())?
            .join("preferences.json");
        // A damaged existing file must not be silently overwritten.
        let previous_preferences = preferences::load(&path)?;
        let now = chrono::Local::now().fixed_offset();
        let reminder_path = path.with_file_name("reminders.json");
        let mut reminders = reminders::load(&reminder_path, now)?;
        if let Some(previous) = &previous_preferences {
            reminders.advance(previous, now);
        }
        // Schedule edits affect future occurrences, retaining outstanding dates.
        if previous_preferences.is_some() {
            reminders.checked_at = now;
        }
        reminders::save(&reminder_path, &reminders)?;
        let login = app.autolaunch();
        let previous = login
            .is_enabled()
            .map_err(|error| format!("Could not check start at login: {error}"))?;
        let apply_login = |enabled| {
            if enabled {
                login.enable()
            } else {
                login.disable()
            }
            .map_err(|error| format!("Could not update start at login: {error}"))
        };
        if previous != preferences.start_at_login {
            apply_login(preferences.start_at_login)?;
        }
        if let Err(error) = preferences::save(&path, &preferences) {
            if previous != preferences.start_at_login {
                apply_login(previous).map_err(|rollback| format!("{error}. {rollback}"))?;
            }
            return Err(error);
        }
        session.gate = None;
        Ok(preferences)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(SettingsLock::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_preferences,
            save_preferences,
            reminder_runtime::load_reminder,
            reminder_runtime::open_timesheet,
            reminder_runtime::complete_reminder,
            reminder_runtime::snooze_reminder
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            app.handle().plugin(
                tauri_plugin_autostart::Builder::new()
                    .arg("--background")
                    .app_name(app.config().identifier.clone())
                    .build(),
            )?;
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let menu = tauri::menu::Menu::with_items(
                app,
                &[
                    &tauri::menu::MenuItem::with_id(
                        app,
                        "pending",
                        "Pending reminders",
                        true,
                        None::<&str>,
                    )?,
                    &tauri::menu::MenuItem::with_id(
                        app,
                        "settings",
                        "Settings",
                        true,
                        None::<&str>,
                    )?,
                    &tauri::menu::PredefinedMenuItem::separator(app)?,
                    &tauri::menu::MenuItem::with_id(app, "quit", "Quit BYT", true, None::<&str>)?,
                ],
            )?;
            let mut tray = tauri::tray::TrayIconBuilder::new()
                .tooltip("BYT — Book Your Time")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "pending" => reminder_runtime::show(app, false),
                    "settings" => reminder_runtime::show(app, true),
                    "quit" => app.exit(0),
                    _ => {}
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            if std::env::args().any(|argument| argument == "--background") {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide()?;
                }
            }
            reminder_runtime::start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
