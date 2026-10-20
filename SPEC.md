# BYT (Book Your Time) — First-Version Specification

Status: agreed product scope. Project scaffold initialized; product features are
not yet implemented.

## Goal

Help people remember to submit their working hours in an existing company
timesheet system. Reminders persist until the user explicitly confirms submission,
with unfinished dates carried into the next scheduled reminder.

The initial audience is the project owner and a small group of colleagues using
Windows and macOS.

## Platform and data

- Build a desktop app using Tauri and Next.js.
- Support Windows and macOS with separate builds.
- Store settings and reminder state locally on each computer.
- Scheduling and reminder state work offline. Opening the timesheet relies on the
  timesheet site's availability.
- No account, backend, synchronization, or manager dashboard in the first version.

## Setup and background operation

- First launch asks for the timesheet URL and reminder schedule.
- Reminders become active after setup is complete.
- Start automatically at login by default, with a setting to disable this.
- Keep the app running in the system tray on Windows or menu bar on macOS when
  its window closes.
- Provide access to settings, pending reminders, and an explicit Quit action.

## Scheduling

- Each person configures one active repeating rule on their computer.
- A weekly rule specifies selected weekdays and a time, such as Friday at 17:00.
- A monthly rule specifies a day of the month and a time.
- Monthly scheduling includes an explicit **Last day of the month** option.
- Interpret scheduled times in the computer's current local time, including when
  the user travels.
- Changing the schedule affects future reminders and preserves already
  outstanding dates.

## Reminder behavior

- When due, show a reminder window that requests the user's attention.
- Repeat every five minutes by default. Make this interval configurable.
- Closing the window leaves the reminder pending.
- Offer snooze durations of 15 minutes, 30 minutes, and one hour.
- Snoozing postpones reminders without completing them. Preserve the snooze
  deadline across app restarts.
- Repeat during the scheduled day unless the user completes or snoozes the
  reminder.
- Once the scheduled day ends, retain unconfirmed dates and present them again at
  the next scheduled occurrence.
- At the next occurrence, show previously outstanding dates alongside the current
  scheduled date, explaining that earlier submissions remain unconfirmed.

## Sleep, shutdown, and missed reminders

- On startup or resume, check whether a scheduled reminder was missed.
- If it is still the scheduled day, show the missed reminder immediately, subject
  to an active snooze.
- If the scheduled day has passed, carry that date into the next scheduled
  reminder instead of interrupting on an unrelated day.
- Combine multiple outstanding dates into one reminder window.
- Never automatically mark an outstanding date as completed.

## Opening the timesheet and completing reminders

- Provide an **Open timesheet** button that opens the configured URL in the
  user's default browser.
- Provide a separate **I've submitted my hours** button.
- Keep completion disabled until the app successfully opens the link and at least
  five seconds have elapsed.
- If opening the link fails, keep completion disabled.
- Opening the timesheet alone does not complete a reminder.
- Confirming submission completes all outstanding dates listed in that reminder.
- Remember completed reminder state so completed dates do not reappear.

The five-second delay is measured after successfully handing the URL to the
browser. It does not verify browser viewing time or actual timesheet submission.

## History

Do not provide a browsable completion history in the first version. Show
outstanding dates in reminders and clear them when submission is confirmed.
Persist the state needed to restore pending reminders and avoid repeating
completed ones.
