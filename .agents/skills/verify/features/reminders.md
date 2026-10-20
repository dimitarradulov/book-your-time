# Respond to outstanding reminders

A due reminder lists every unconfirmed date. The user opens their timesheet, separately confirms submission after the required delay, or snoozes without clearing outstanding dates.

## Sub-features

- `reminders.due` combines earlier unconfirmed dates with the current occurrence in one attention-requesting window.
- `reminders.open` hands the configured URL to the default browser without clearing dates.
- `reminders.gate` enables confirmation only after successful URL handoff and at least five elapsed seconds.
- `reminders.confirm` clears all listed dates and remembers completion across restarts.
- `reminders.snooze` offers 15 minutes, 30 minutes, and one hour and persists the deadline.
- `reminders.repeat` repeats at the configured interval while the scheduled day continues.

## How to get to it (user POV)

- Wait for a scheduled occurrence, or resume/start BYT on the same scheduled day after a missed occurrence.
- Leave a due reminder unconfirmed and wait for its configured repeat interval.
- Open pending reminders through the tray or menu bar.
- Carry an older unconfirmed date into the next scheduled occurrence.

## Driving it with Computer Use

Preconditions: An owned configured app with a due occurrence and at least one earlier outstanding date. Currently blocked because no scheduler, reminder state, or reminder UI exists. There is no supported seed or disposable time control. Report each unmet precondition.

- Inspect. Fetch the native tree and screenshot. Require `Time to submit your hours`, all actual outstanding dates, and explanation that earlier dates remain unconfirmed. Confirm one combined window. The mock's `16 Oct 2026` and `23 Oct 2026` are illustrative.
- Gate before opening. Require `I've submitted my hours` disabled. Wait at least five seconds without opening and require it still disabled. This distinguishes the browser-handoff gate from a delay measured at reminder creation.
- Open. Click the actual `Open timesheet` control via its freshly observed accessibility index. Observe the default browser reaching the configured disposable loopback URL and corroborate the HTTP request. Require dates to remain pending. Record the successful handoff time, confirmation still disabled before five seconds, and enabled at or after five seconds.
- Failure. At an existing production URL-opening boundary, induce handoff failure. Require a visible failure state and disabled confirmation even after five seconds. An unreachable website after a successful browser handoff is not a handoff failure. If no controllable boundary exists, record this branch blocked.
- Confirm. Click `I've submitted my hours` after the gate enables it. Require every date shown in that reminder to disappear and the status view to show the next occurrence. Inspect persisted completion state read-only, quit and reopen, and require completed dates to stay absent.
- Snooze. On a separate pending reminder, choose each supported duration through the observed snooze control. Require pending dates to remain, no reminders before the deadline, and the same deadline after restart. Observe the reminder again when eligible. Never complete the reminder to make snooze pass.
- Repeat and close. Close an unconfirmed window through its native close button. Require the app to stay running, dates to remain pending, and a reminder to reappear after the selected interval during that scheduled day. At the next occurrence, require older unconfirmed dates alongside the current one.

## Gotchas

- No real user action in this checkout reaches these states. A screenshot of the mock or injected UI state cannot prove them.
- A five-second observation needs timestamps around both handoff and control state. A single enabled-button screenshot cannot prove the gate.
- Snooze, completion, repeat, and reopen are separate branches. Record all entry points and branches covered; one confirmation pass does not cover snooze.
- BYT confirms the user's declaration. It never verifies or submits hours in the external timesheet system.
- Waiting across days is currently required for carryover without a disposable clock. Do not alter the user's system clock or mark this branch passed via internal state injection.
