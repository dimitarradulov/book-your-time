# Edit reminder preferences

The compact settings view changes the timesheet URL, repeating schedule, repeat interval, and login preference. Saving affects future reminders while retaining outstanding dates.

## Sub-features

- `settings.open` opens from the small settings control in status and reminder views.
- `settings.rule` edits weekly/monthly scheduling and the timesheet URL.
- `settings.interval` edits the repeat interval, which defaults to five minutes.
- `settings.login` changes start-at-login, which defaults on.
- `settings.save` persists edits and preserves existing pending dates.
- `settings.cancel` and `settings.back` leave without accidentally saving draft changes.

## How to get to it (user POV)

- Use the settings icon in the status view.
- Use the settings icon in a due reminder with outstanding dates.
- The mock also shows a settings icon on setup and within settings. Discover its actual behavior when implemented and record any additional working entry point.
- Leave through `Save changes`, `Cancel`, or `Back`.

## Driving it with Computer Use

Preconditions: An owned configured app and, for pending-date preservation, a real outstanding reminder. Currently blocked because no settings control, saved values, or settings page exists.

- Open. Read a fresh native accessibility tree, find the actual settings control, and click its current index. Require the `Settings` heading and populated `Timesheet URL`, `Reminder schedule`, `Time`, `Repeat reminder every`, and `Start at login` controls. Record a missing accessible name on the icon as a driver/accessibility defect.
- Edit. Change the URL to a disposable loopback destination, switch weekly/monthly mode, change weekdays or the last-day selection, time, and repeat interval through actual controls. Capture draft state. Exercise both schedule modes through [setup](setup.md)'s rule checks.
- Cancel. Click `Cancel`. Reopen from the same entry point and require the original saved values and pending dates. Repeat a draft edit and use `Back`; require draft changes not to become persisted settings. If the implementation intentionally gives Back another contract, resolve that product decision before asserting it.
- Save. Apply edits and click `Save changes`. Require return to the previous usable view and an updated future occurrence. Read the actual persisted values, quit and reopen, and require every saved preference to survive. Existing pending dates must remain intact.
- Login. In a disposable OS profile, toggle `Start at login`, save, and inspect actual OS login registration. Require registration to follow the saved setting after restart. Restore only this run's owned registration. A toggle screenshot alone does not prove autostart.
- Coverage. Repeat open/save/cancel from both status and a pending reminder. Ensure navigating to settings does not silently confirm dates or reset an existing snooze deadline.

## Gotchas

- The mock's gear icons have no confirmed implemented accessibility labels. Obtain real handles from the app before driving them.
- SPEC.md defines preserved outstanding dates and configurable intervals. It does not list all interval choices or define unsaved-change behavior for Back. Treat unresolved behavior as a product question, not a passing assumption.
- Persistence and login registration have no implementation or known paths yet. Discover them before any mutation test and verify disposal is isolated.
- The timesheet URL in the mock is illustrative. Use verification-owned data.
