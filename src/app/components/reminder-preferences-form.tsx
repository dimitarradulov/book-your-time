"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  defaultPreferences, errorMessage, loadPreferences, savePreferences,
  type Preferences, type Schedule,
} from "../lib/preferences";

const weekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const monthDays = Array.from({ length: 31 }, (_, index) => String(index + 1));
type LoadedPreferences =
  | { kind: "loading" }
  | { kind: "ready"; preferences: Preferences | null }
  | { kind: "error"; message: string };

export function ReminderPreferencesForm({ variant }: { variant: "setup" | "settings" }) {
  const [loaded, setLoaded] = useState<LoadedPreferences>({ kind: "loading" });
  const [attempt, setAttempt] = useState(0);
  const router = useRouter();

  useEffect(() => {
    let cancelled = false;
    loadPreferences().then(
      (preferences) => { if (!cancelled) setLoaded({ kind: "ready", preferences }); },
      (error: unknown) => { if (!cancelled) setLoaded({ kind: "error", message: errorMessage(error) }); },
    );
    return () => { cancelled = true; };
  }, [attempt]);

  const configured = loaded.kind === "ready" && loaded.preferences !== null;
  return (
    <>
      {variant === "setup" ? (
        <header className="setup-header">
          <div className="heading-row">
            <h1>{configured ? "Your reminder settings" : "Set up your reminders"}</h1>
            <Link className="settings-link" href="/settings/" aria-label="Settings" title="Settings">
              <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                <path d="m9.5 3-.6 2.1-1.7 1-2.2-.5-2.5 4.3L4 11.5v2L2.5 15l2.5 4.3 2.2-.5 1.7 1 .6 2.2h5l.6-2.2 1.7-1 2.2.5 2.5-4.3-1.5-1.5v-2l1.5-1.6L19 5.6l-2.2.5-1.7-1-.6-2.1Z" />
                <circle cx="12" cy="12.5" r="3.2" />
              </svg>
            </Link>
          </div>
          <p>{configured ? "Saved on this computer." : "Choose where you submit your hours and when to be reminded."}</p>
        </header>
      ) : null}
      {loaded.kind === "loading" ? <p role="status">Loading settings…</p> : null}
      {loaded.kind === "error" ? (
        <div role="alert">
          <p className="error">{loaded.message}</p>
          <button className="secondary-button" onClick={() => {
            setLoaded({ kind: "loading" });
            setAttempt((value) => value + 1);
          }}>Try again</button>
        </div>
      ) : null}
      {loaded.kind === "ready" ? (
        variant === "setup" && loaded.preferences ? (
          <div role="status">
            <div className="form-row"><span>Timesheet URL</span><p className="saved-url">{loaded.preferences.url}</p></div>
            <div className="form-row"><span>Reminder schedule</span><p>
              {loaded.preferences.schedule.kind === "weekly"
                ? loaded.preferences.schedule.days.map((day) => weekdays[day]).join(", ")
                : loaded.preferences.schedule.day === "last" ? "Last day of the month" : `Day ${loaded.preferences.schedule.day} of the month`}
              {" at "}{loaded.preferences.time}
            </p></div>
            <p className="field-note">Uses your computer’s local time. Reminders are active.</p>
            <Link className="secondary-button" href="/settings/">Edit settings</Link>
          </div>
        ) : (
          <PreferencesFields variant={variant} initial={loaded.preferences ?? defaultPreferences}
            onSaved={(preferences) => {
              if (variant === "settings") router.push("/");
              else setLoaded({ kind: "ready", preferences });
            }} />
        )
      ) : null}
    </>
  );
}

function urlError(value: string): string | undefined {
  if (!value.trim()) return "Enter your timesheet URL.";
  if (!/^https?:\/\//i.test(value.trim())) {
    return "Use a full web address starting with https:// or http://.";
  }
  try {
    const url = new URL(value.trim());
    if (!["https:", "http:"].includes(url.protocol) || !url.hostname) {
      return "Use a full web address starting with https:// or http://.";
    }
    if (url.username || url.password) return "Use a URL without a username or password in it.";
  } catch {
    return "Use a full web address starting with https:// or http://.";
  }
}

function PreferencesFields({ variant, initial, onSaved }: {
  variant: "setup" | "settings";
  initial: Preferences;
  onSaved: (preferences: Preferences) => void;
}) {
  const [url, setUrl] = useState(initial.url);
  const [kind, setKind] = useState<Schedule["kind"]>(initial.schedule.kind);
  const [weeklyDays, setWeeklyDays] = useState(initial.schedule.kind === "weekly" ? initial.schedule.days : [4]);
  const [monthlyDay, setMonthlyDay] = useState(initial.schedule.kind === "monthly" ? String(initial.schedule.day) : "last");
  const [time, setTime] = useState(initial.time);
  const [repeatInterval, setRepeatInterval] = useState(String(initial.repeatInterval));
  const [startAtLogin, setStartAtLogin] = useState(initial.startAtLogin);
  const [submitted, setSubmitted] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const savingRef = useRef(false);
  const formRef = useRef<HTMLFormElement>(null);
  const schedule: Schedule = kind === "weekly" ? { kind, days: weeklyDays } : { kind, day: monthlyDay === "last" ? "last" : Number(monthlyDay) };
  const errors = {
    url: urlError(url),
    days: schedule.kind === "weekly" && schedule.days.length === 0 ? "Choose at least one weekday." : undefined,
    time: /^([01]\d|2[0-3]):[0-5]\d$/.test(time) ? undefined : "Choose a reminder time.",
  };

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (savingRef.current) return;
    setSaveError(null);
    setSubmitted(true);
    if (Object.values(errors).some(Boolean)) {
      formRef.current?.querySelector<HTMLElement>(
        errors.url ? "#timesheet-url" : errors.days ? "input[name=weekday]" : "#reminder-time",
      )?.focus();
      return;
    }
    savingRef.current = true;
    setSaving(true);
    try {
      const saved = await savePreferences({
        url: url.trim(), schedule, time,
        repeatInterval: Number(repeatInterval), startAtLogin,
      });
      onSaved(saved);
    } catch (error: unknown) {
      setSaveError(errorMessage(error));
    } finally {
      savingRef.current = false;
      setSaving(false);
    }
  }

  return (
    <form ref={formRef} onSubmit={save} noValidate aria-busy={saving}>
      <fieldset className="preferences-fields" disabled={saving}>
      <div className="form-row url-row">
        <label htmlFor="timesheet-url">Timesheet URL</label>
        <div className="field">
          <input id="timesheet-url" type="url" autoComplete="url" spellCheck={false}
            placeholder="https://timesheets.example.com" value={url} required
            aria-invalid={submitted && Boolean(errors.url)}
            aria-describedby={submitted && errors.url ? "url-error" : undefined}
            onChange={(event) => setUrl(event.target.value)} />
          {submitted && errors.url ? <p className="error" id="url-error">{errors.url}</p> : null}
        </div>
      </div>
      <fieldset className="form-row schedule-row">
        <legend>Reminder schedule</legend>
        <div className="segmented-control">
          <label><input type="radio" name="schedule" value="weekly" checked={kind === "weekly"}
            onChange={() => setKind("weekly")} /><span>Weekly</span></label>
          <label><input type="radio" name="schedule" value="monthly" checked={kind === "monthly"}
            onChange={() => setKind("monthly")} /><span>Monthly</span></label>
        </div>
      </fieldset>
      {schedule.kind === "weekly" ? (
        <fieldset className="form-row weekday-row" aria-describedby={submitted && errors.days ? "days-error" : undefined}>
          <legend>Weekdays</legend>
          <div className="field">
            <div className="weekdays">
              {weekdays.map((day, index) => (
                <label key={day} title={day}>
                  <input type="checkbox" name="weekday" aria-label={day} checked={schedule.days.includes(index)}
                    onChange={(event) => {
                      const selected = event.target.checked;
                      setWeeklyDays((days) => selected ? [...days, index].sort() : days.filter((value) => value !== index));
                    }} /><span aria-hidden="true">{day.slice(0, 3)}</span>
                </label>
              ))}
            </div>
            {submitted && errors.days ? <p className="error" id="days-error">{errors.days}</p> : null}
          </div>
        </fieldset>
      ) : (
        <div className="form-row">
          <label htmlFor="month-day">Day of month</label>
          <div className="field">
            <select id="month-day" value={String(schedule.day)} onChange={(event) => setMonthlyDay(event.target.value)}>
              {monthDays.map((day) => <option key={day} value={day}>{day}</option>)}
              <option value="last">Last day of the month</option>
            </select>
            {Number(schedule.day) > 28 ? <p className="field-note">For every month, choose “Last day of the month”.</p> : null}
          </div>
        </div>
      )}
      <div className="form-row time-row">
        <label htmlFor="reminder-time">Time</label>
        <div className="field">
          <input id="reminder-time" type="time" value={time} required
            aria-invalid={submitted && Boolean(errors.time)} aria-describedby={submitted && errors.time ? "time-error time-note" : "time-note"}
            onChange={(event) => setTime(event.target.value)} />
          {submitted && errors.time ? <p className="error" id="time-error">{errors.time}</p> : null}
          <p className="field-note" id="time-note">Uses your computer’s local time.</p>
        </div>
      </div>
      {variant === "settings" ? (
        <div className="settings-preferences">
          <div className="form-row">
            <label htmlFor="repeat-interval">Repeat reminder every</label>
            <select id="repeat-interval" value={repeatInterval}
              onChange={(event) => setRepeatInterval(event.target.value)}>
              {[1, 5, 10, 15, 30, 60].map((minutes) => (
                <option key={minutes} value={String(minutes)}>{minutes} {minutes === 1 ? "minute" : "minutes"}</option>
              ))}
            </select>
          </div>
          <div className="form-row login-row">
            <label htmlFor="start-at-login">Start at login</label>
            <input id="start-at-login" className="switch" type="checkbox" role="switch"
              checked={startAtLogin} onChange={(event) => setStartAtLogin(event.target.checked)} />
          </div>
        </div>
      ) : null}
      <footer className="setup-footer">
        {variant === "settings" ? (
          <>
            <div className="settings-actions">
              <button className="primary-button" type="submit">{saving ? "Saving…" : "Save changes"}</button>
              <Link className="secondary-button" href="/" aria-disabled={saving} onClick={(event) => { if (saving) event.preventDefault(); }}>Cancel</Link>
            </div>

          </>
        ) : (
          <>
            <button className="primary-button" type="submit">{saving ? "Saving…" : "Save setup"}</button>
            <p className="preview-note">Start at login is on by default. You can change it in Settings.</p>
          </>
        )}
      </footer>
      </fieldset>
      {saveError ? <p className="error" role="alert">{saveError} Your changes haven’t been saved. Try again.</p> : null}
    </form>
  );
}
