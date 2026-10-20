# See the next reminder and keep BYT running

When there are no currently presented reminders, BYT shows its next occurrence and schedule. Closing its window keeps it available in the tray or menu bar until the user explicitly quits.

## Sub-features

- `status.next` shows the next reminder and readable schedule summary.
- `status.open` opens the configured timesheet outside a due reminder.
- `background.close` closes the window while the app keeps running.
- `background.reopen` opens settings and pending reminders through the tray or menu bar.
- `background.missed` handles startup, sleep, resume, and carried dates without completing them.
- `background.quit` explicitly exits the app.

## How to get to it (user POV)

- Finish initial setup, confirm a reminder, or reopen a configured app with no currently presented reminder.
- Close the native window and use the system tray on Windows or menu bar on macOS.
- Start or resume the app after a missed occurrence, both during and after the scheduled day.

## Driving it with Computer Use

Preconditions: A configured owned native app, known active rule, and disposable persisted state. Currently blocked because status, scheduling, and tray/menu bar operation are unimplemented. The scaffold's ordinary native menu does not prove a tray menu.

- Inspect status. Require `You're all set`, the next local date and time, and a schedule summary matching the saved rule. Capture actual labels even if punctuation differs. Independently calculate the next occurrence for the selected rule and timezone. Require no outstanding date to be cleared by merely showing status.
- Open timesheet. Click the status view's `Open timesheet`. Observe the default browser and disposable HTTP destination. Require schedule and pending state to remain unchanged.
- Close and reopen. Use the observed native close button. Check the owned PID remains alive and no window remains visible. Inspect the actual menu bar or system tray through the platform UI driver, then use its status/pending-reminder or settings entry to reopen. Capture the menu, action, resulting view, and process evidence.
- Missed today. Configure through the UI before an upcoming occurrence, quit, and reopen later on the same scheduled day. Require an immediate combined reminder unless a persisted snooze is active. Repeat through OS sleep/resume when that operation is isolated from the user's session.
- Missed earlier. Start after the scheduled day ends. Require no interruption on that unrelated day. At the next scheduled occurrence, require all missed/unconfirmed dates together. Exercise an active snooze across restart as a separate branch.
- Quit. Select the app's explicit `Quit` action. Require the owned PID to exit and the tray/menu bar item to disappear. Reopen and corroborate preserved settings and pending dates.

## Gotchas

- The current scaffold has no close-to-tray handler. Closing the starter window cannot pass `background.close`.
- Scheduling uses the computer's current local time, including travel. Timezone changes and sleep need an isolated OS session; otherwise report those branches blocked.
- A browser tab at `http://127.0.0.1:3000` cannot establish native background behavior or lifecycle.
- Never quit a user-owned BYT process. The helper's exact PID and unique app identity define this run.
