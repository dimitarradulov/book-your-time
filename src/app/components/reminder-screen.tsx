"use client";

import Link from "next/link";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { errorMessage } from "../lib/preferences";
import { loadReminder, type ReminderSnapshot } from "../lib/reminders";
import { ReminderPreferencesForm } from "./reminder-preferences-form";

type Screen = { kind: "loading" } | { kind: "ready"; snapshot: ReminderSnapshot } | { kind: "error"; message: string };
const dateFormatter = new Intl.DateTimeFormat(undefined, { day: "numeric", month: "short", year: "numeric" });
const dateTimeFormatter = new Intl.DateTimeFormat(undefined, { weekday: "short", day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
function dateLabel(date: string) { return dateFormatter.format(new Date(`${date}T12:00:00`)); }

export function ReminderScreen() {
  const [screen, setScreen] = useState<Screen>({ kind: "loading" });
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [reviewPending, setReviewPending] = useState(false);
  const [snooze, setSnooze] = useState("15");
  const [attempt, setAttempt] = useState(0);
  const actionLock = useRef(false);
  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    async function refresh() {
      try {
        const snapshot = await loadReminder();
        if (!cancelled && !actionLock.current) setScreen({ kind: "ready", snapshot });
      } catch (cause: unknown) {
        if (!cancelled) setScreen({ kind: "error", message: errorMessage(cause) });
      }
      if (!cancelled) timer = setTimeout(refresh, 1000);
    }
    void refresh();
    return () => { cancelled = true; clearTimeout(timer); };
  }, [attempt]);

  async function act(command: string, args?: { minutes: number } | { dates: string[] }) {
    if (actionLock.current) return;
    actionLock.current = true;
    setBusy(true);
    setError(null);
    try {
      const snapshot = await invoke<ReminderSnapshot>(command, args);
      if (command === "snooze_reminder") setReviewPending(false);
      setScreen({ kind: "ready", snapshot });
    } catch (cause: unknown) {
      if (command === "open_timesheet") {
        setScreen((current) => current.kind === "ready"
          ? { kind: "ready", snapshot: { ...current.snapshot, confirmationSeconds: null } }
          : current);
      }
      setError(errorMessage(cause));
    } finally {
      actionLock.current = false;
      setBusy(false);
    }
  }

  if (screen.kind === "loading") return <p role="status">Loading reminders…</p>;
  if (screen.kind === "error") return <div role="alert"><h1>Couldn’t load reminders</h1><p className="error">{screen.message}</p><button className="secondary-button" onClick={() => setAttempt((value) => value + 1)}>Try again</button></div>;
  const { snapshot } = screen;
  if (!snapshot.preferences) return <ReminderPreferencesForm variant="setup" onSaved={() => setAttempt((value) => value + 1)} />;
  const pending = snapshot.pending.length > 0;
  const snoozed = snapshot.snoozed;
  const showReminder = pending && (!snoozed || reviewPending);
  const seconds = snapshot.confirmationSeconds;
  return <>
    <header className="setup-header">
      <div className="heading-row">
        <h1>{showReminder ? "Time to submit your hours" : snoozed ? "Your reminder is snoozed" : "You’re all set"}</h1>
        <Link className="settings-link" href="/settings/" aria-label="Settings" title="Settings">
          <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="m9.5 3-.6 2.1-1.7 1-2.2-.5-2.5 4.3L4 11.5v2L2.5 15l2.5 4.3 2.2-.5 1.7 1 .6 2.2h5l.6-2.2 1.7-1 2.2.5 2.5-4.3-1.5-1.5v-2l1.5-1.6L19 5.6l-2.2.5-1.7-1-.6-2.1Z" /><circle cx="12" cy="12.5" r="3.2" /></svg>
        </Link>
      </div>
      {showReminder ? <p>{snapshot.pending.length === 1 ? "One reminder date is still unconfirmed." : `${snapshot.pending.length} reminder dates are still unconfirmed.`}</p> : null}
    </header>
    {snapshot.missingMonths.length > 0 ? <section className="missing-dates" aria-label="Missing booking dates">
      <h2>Set your booking {snapshot.missingMonths.length === 1 ? "date" : "dates"}</h2>
      <p>No date chosen for {snapshot.missingMonths.map((month) => new Intl.DateTimeFormat(undefined, { month: "long", year: "numeric" }).format(new Date(`${month}T12:00:00`))).join(" or ")}. BYT will remind you once you choose a date.</p>
      <Link className="secondary-button" href="/settings/">Set booking dates</Link>
    </section> : null}
    {showReminder ? <>
      <ul className="pending-dates" aria-label="Unconfirmed reminder dates">
        {snapshot.pending.map((date) => <li key={date}><time dateTime={date}>{dateLabel(date)}</time><span>{date === snapshot.today ? "Today" : "Earlier reminder"}</span></li>)}
      </ul>
      {snapshot.pending.some((date) => date < snapshot.today) ? <p className="field-note">Earlier submissions remain unconfirmed.</p> : null}
      <div className="reminder-actions" aria-busy={busy}>
        <button className="primary-button" disabled={busy} onClick={() => void act("open_timesheet", { dates: snapshot.pending })}>Open timesheet</button>
        <div>
          <button className="secondary-button confirmation-button" disabled={busy || seconds !== 0} aria-describedby="confirmation-note" onClick={() => void act("complete_reminder", { dates: snapshot.pending })}>I&apos;ve submitted my hours</button>
          <p className="field-note" id="confirmation-note">{seconds === null ? "Available 5 seconds after opening the timesheet." : seconds > 0 ? `Available in ${seconds} ${seconds === 1 ? "second" : "seconds"}.` : "Ready when you’ve submitted your hours."}</p>
        </div>
      </div>
      <div className="reminder-footer">
        <div className="snooze-actions">
          <select aria-label="Snooze duration" value={snooze} disabled={busy} onChange={(event) => setSnooze(event.target.value)}>
            <option value="15">15 minutes</option><option value="30">30 minutes</option><option value="60">1 hour</option>
          </select>
          <button className="secondary-button" disabled={busy} onClick={() => void act("snooze_reminder", { minutes: Number(snooze) })}>Snooze</button>
        </div>
        <p className="field-note">Confirming submission clears {snapshot.pending.length === 1 ? "this date" : `all ${snapshot.pending.length} dates`}.</p>
      </div>
    </> : <>
      <section className="next-reminder" aria-label="Next reminder">
        <h2>{snoozed && snapshot.snoozeUntil ? `Snoozed until ${dateTimeFormatter.format(new Date(snapshot.snoozeUntil))}` : snapshot.nextReminder ? `Next reminder: ${dateTimeFormatter.format(new Date(snapshot.nextReminder))}` : "No upcoming reminder"}</h2>
        <p>{snoozed ? "Your unconfirmed dates are saved." : "Uses your computer’s local time."}</p>
      </section>
      <div className="status-actions">
        <button className="secondary-button" disabled={busy} onClick={() => void act("open_timesheet", { dates: snapshot.pending })}>Open timesheet</button>
        {snoozed ? <button className="secondary-button" onClick={() => setReviewPending(true)}>Review pending dates</button> : null}
      </div>
    </>}
    {error ? <p className="error" role="alert">{error} Please try again.</p> : null}
    <p className="background-note">BYT keeps running when you close this window.</p>
  </>;
}
