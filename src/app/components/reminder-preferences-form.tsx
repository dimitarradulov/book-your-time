"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState, type FormEvent } from "react";
import {
  defaultPreferences, errorMessage, savePreferences,
  type Preferences, type Schedule,
} from "../lib/preferences";

import { loadReminder } from "../lib/reminders";

const weekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
function localDate(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}
function legacyDate(preferences: Preferences): string[] {
  if (preferences.schedule.kind !== "legacymonthly") return [];
  const now = new Date();
  for (let offset = 0; offset < 3; offset++) {
    const month = new Date(now.getFullYear(), now.getMonth() + offset, 1);
    const day = preferences.schedule.day === "last"
      ? new Date(month.getFullYear(), month.getMonth() + 1, 0).getDate() : preferences.schedule.day;
    const candidate = new Date(month.getFullYear(), month.getMonth(), day);
    const [hours, minutes] = preferences.time.split(":").map(Number);
    candidate.setHours(hours, minutes, 0, 0);
    if (candidate.getMonth() === month.getMonth() && candidate > now) return [localDate(candidate)];
  }
  return [];
}
type LoadedPreferences =
  | { kind: "loading" }
  | { kind: "ready"; preferences: Preferences | null; lockedBookingDates: string[] }
  | { kind: "error"; message: string };

export function ReminderPreferencesForm({ variant, onSaved }: { variant: "setup" | "settings"; onSaved?: () => void }) {
  const [loaded, setLoaded] = useState<LoadedPreferences>({ kind: "loading" });
  const [attempt, setAttempt] = useState(0);
  const router = useRouter();

  useEffect(() => {
    let cancelled = false;
    loadReminder().then(
      ({ preferences, lockedBookingDates }) => { if (!cancelled) setLoaded({ kind: "ready", preferences, lockedBookingDates }); },
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
                : loaded.preferences.schedule.kind === "monthly" ? `${loaded.preferences.schedule.dates.length} booking dates chosen` : "Previous monthly rule"}
              {" at "}{loaded.preferences.time}
            </p></div>
            <p className="field-note">Uses your computer’s local time. Reminders are active.</p>
            <Link className="secondary-button" href="/settings/">Edit settings</Link>
          </div>
        ) : (
          <PreferencesFields variant={variant} initial={loaded.preferences ?? defaultPreferences} occurredDates={loaded.lockedBookingDates}
            onSaved={(preferences) => {
              if (variant === "settings") router.push("/");
              else if (onSaved) onSaved();
              else setLoaded({ kind: "ready", preferences, lockedBookingDates: loaded.lockedBookingDates });
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

function PreferencesFields({ variant, initial, occurredDates, onSaved }: {
  variant: "setup" | "settings";
  initial: Preferences;
  occurredDates: string[];
  onSaved: (preferences: Preferences) => void;
}) {
  const [url, setUrl] = useState(initial.url);
  const [kind, setKind] = useState<"weekly" | "monthly">(initial.schedule.kind === "weekly" ? "weekly" : "monthly");
  const [weeklyDays, setWeeklyDays] = useState(initial.schedule.kind === "weekly" ? initial.schedule.days : [4]);
  const [monthlyDates, setMonthlyDates] = useState<string[]>(initial.schedule.kind === "monthly" ? initial.schedule.dates : legacyDate(initial));
  const [today] = useState(() => localDate(new Date()));
  const [openedAt] = useState(() => new Date());
  const lockedDates = initial.schedule.kind === "monthly"
    ? initial.schedule.dates.filter((date) => occurredDates.includes(date) || new Date(`${date}T${initial.time}:00`) <= openedAt) : [];
  const [time, setTime] = useState(initial.time);
  const [repeatInterval, setRepeatInterval] = useState(String(initial.repeatInterval));
  const [startAtLogin, setStartAtLogin] = useState(initial.startAtLogin);
  const [reminderSound, setReminderSound] = useState(initial.reminderSound);
  const [submitted, setSubmitted] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const savingRef = useRef(false);
  const formRef = useRef<HTMLFormElement>(null);
  const schedule: Schedule = kind === "weekly" ? { kind, days: weeklyDays } : { kind, dates: monthlyDates.filter(Boolean).sort() };
  const months = schedule.kind === "monthly" ? schedule.dates.map((date) => date.slice(0, 7)) : [];
  const datesError = schedule.kind === "monthly" && new Set(months).size !== months.length
    ? "Choose at most one booking date per month."
    : schedule.kind === "monthly" && schedule.dates.some((date) => date < today && !(initial.schedule.kind === "monthly" && initial.schedule.dates.includes(date)))
      ? "New booking dates must be today or later." : undefined;
  const errors = {
    url: urlError(url),
    dates: datesError,
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
        errors.url ? "#timesheet-url" : errors.days ? "input[name=weekday]" : errors.dates ? "input[type=date]" : "#reminder-time",
      )?.focus();
      return;
    }
    savingRef.current = true;
    setSaving(true);
    try {
      const saved = await savePreferences({
        url: url.trim(), schedule, time,
        repeatInterval: Number(repeatInterval), startAtLogin, reminderSound,
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
        <div className="form-row monthly-row">
          <span>Booking dates</span>
          <div className="field">
            {initial.schedule.kind === "legacymonthly" ? <p className="field-note">Your existing monthly rule stays active until you save. Saving switches to individual booking dates, starting with the next date shown below. Outstanding reminders stay saved.</p> : null}
            <div className="monthly-dates">
              {monthlyDates.map((date, index) => {
                const locked = lockedDates.includes(date);
                return <div className="monthly-date" key={index}>
                  <label htmlFor={`booking-date-${index}`}>Booking date {index + 1}</label>
                  <div className="monthly-date-actions">
                    <input id={`booking-date-${index}`} type="date" value={date} min={locked ? undefined : today}
                      readOnly={locked} aria-describedby={locked ? `locked-date-${index}` : "dates-note"}
                      aria-invalid={submitted && Boolean(errors.dates)}
                      onChange={(event) => setMonthlyDates((dates) => dates.map((value, at) => at === index ? event.target.value : value))} />
                    {!locked ? <button type="button" className="secondary-button" aria-label={`Remove booking date ${index + 1}`} onClick={() => setMonthlyDates((dates) => dates.filter((_, at) => at !== index))}>Remove</button> : null}
                  </div>
                  {locked ? <p className="field-note" id={`locked-date-${index}`}>Reminder already due. This date is locked.</p> : null}
                </div>;
              })}
            </div>
            <button type="button" className="secondary-button" onClick={() => setMonthlyDates((dates) => [...dates, ""])}>Add booking date</button>
            <p className="field-note" id="dates-note">Choose one date per month as dates are announced. You can save without any dates.</p>
            {submitted && errors.dates ? <p className="error" role="alert">{errors.dates}</p> : null}
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
          <p className="field-note" id="time-note">{kind === "monthly" ? "Shared by all booking dates. " : ""}Uses your computer’s local time.</p>
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
            <label htmlFor="reminder-sound">Reminder sound</label>
            <input id="reminder-sound" className="switch" type="checkbox" role="switch"
              checked={reminderSound} onChange={(event) => setReminderSound(event.target.checked)} />
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
