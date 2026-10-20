import Link from "next/link";
import { ReminderPreferencesForm } from "../components/reminder-preferences-form";

export default function Settings() {
  return (
    <main className="setup settings">
      <Link className="back-link" href="/">
        <svg viewBox="0 0 20 20" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <path d="m12 4-6 6 6 6" />
        </svg>
        Back
      </Link>
      <header className="setup-header">
        <h1>Settings</h1>
      </header>
      <ReminderPreferencesForm variant="settings" />
    </main>
  );
}
