# Configure reminders on first launch

A new user enters their timesheet URL and one repeating weekly or monthly schedule, then starts reminders in the compact setup view.

## Sub-features

- `setup.url` accepts the company's timesheet URL.
- `setup.weekly` selects one or more weekdays and a local time.
- `setup.monthly` selects a monthly day or `Last day of the month`, with a local time.
- `setup.start` activates reminders only after setup and opens the status view.
- `setup.persist` restores the configuration after quitting and reopening.

## How to get to it (user POV)

- Open BYT for the first time with no saved settings.
- After setup, edit the rule through [settings](settings.md). That separate entry point must also be checked.

## Driving it with Computer Use

Preconditions: An owned instance with empty disposable storage. Currently blocked because the app displays the starter screen and has no setup fields. Capture that actual screen and report the missing `Set up your reminders` view.

- Discover. Read `await sky.get_app_state({app, disableDiff: true})`. When implemented, require the setup heading and actual labeled controls `Timesheet URL`, `Reminder schedule`, and `Time`. Match controls from that tree through `sky.click`, `sky.type_text`, or `sky.set_value`.
- Weekly. Enter a disposable loopback HTTP URL, choose `Weekly`, select Friday, and set `17:00`. Exercise multiple selected weekdays too. Require the selected weekday states and time to match the choices. Record labels that distinguish Tuesday/Thursday and Saturday/Sunday; the mock's repeated initials alone are ambiguous.
- Monthly. Choose `Monthly`, select an ordinary date, then exercise `Last day of the month`. Require the selected mode and date to remain visible. The compact mock shows only weekly mode, so derive monthly behavior from SPEC.md and discover the implemented labels.
- Start. Click `Start reminders`. Require the status view with the next reminder and schedule summary matching the active rule in the machine's local time. Record the actual current date, timezone, and selected rule. Never assert the mock's fixed date.
- Persist. Inspect the actual saved settings read-only, use explicit Quit, and reopen the same disposable app. Require the same rule and status screen without repeating setup. Check login registration defaults on only in a disposable OS profile where that side effect can be removed.

## Gotchas

- There is no settings schema, file path, setup route, URL validation contract, or persistence implementation yet. Record unmet preconditions rather than inventing them.
- A loopback URL is verification data. It must resolve to a server owned by the verification run before testing actual browser opening.
- Monthly dates that do not exist in a month have no specified fallback policy beyond the explicit last-day option. Report that unresolved case rather than imposing a policy.
- Default login registration is an OS side effect. The current scaffold has none. Future runs need disposable OS state before exercising it.
