# BYT verification map

Read this index before driving BYT. The native macOS recipe and executable helper live in [the verify skill](../SKILL.md). Use its owned instance, doctor, evidence directory, and cleanup for every journey.

## Baseline preconditions

The current app is a scaffold. Only `scaffold` is runnable. There is no authentication, configured schedule, reminder seed, state store, or time-control boundary. A fresh verification identifier launches the starter screen. The compact [chosen mock](../../../../.impeccable/mocks/variant-b.png) describes four future views, not implemented routes. [SPEC.md](../../../../SPEC.md) owns behavior and [PRODUCT.md](../../../../PRODUCT.md) records the single-window design decision.

The helper builds a native app with embedded frontend files. Its doctor checks ownership and build identity. UI readiness additionally requires a visible native accessibility tree and screenshot. Verification builds share output directories, so the helper permits one at a time. Existing user instances are never targets.

## Driving conventions

Use Computer Use through `node_repl` and `@oai/sky` on macOS. Read fresh accessibility state before each action. Match visible control names, then use the returned index. Never carry numeric indices between states. The map's future control labels come from the mock or specification; they are not selectors confirmed in the running app.

Every feature file has four sections. Each journey pairs user actions with observable proof. Record every entry point attempted. Passing one entry point does not cover another. Missing controls, seed data, time control, or platform support produce an explicit blocked result, never a weaker assertion.

## Proof and skip reporting

Write a feature ID, entry point, action transcript, before/after accessibility trees and screenshots, and assertions to the run's evidence directory. Inspect real side effects for mutations, then restart through the user path to corroborate persistence. Preserve evidence after cleanup. Report macOS and Windows coverage separately.

## Features

| Feature | Current status | Entry points and scope |
| --- | --- | --- |
| [scaffold](scaffold.md) | Runnable | Native launch and window zoom. Starter content and native rendering. |
| [setup](setup.md) | Blocked, unimplemented | First launch. URL, weekly/monthly schedule, last day, activation, and persistence. |
| [reminders](reminders.md) | Blocked, unimplemented | Due window, repeated reminder, carried dates, and pending-reminder access. Browser handoff, five-second completion gate, confirmation, and snooze. |
| [status-background](status-background.md) | Blocked, unimplemented | Setup completion, confirmation, native reopen, and tray/menu bar. Next occurrence, missed reminders, close-to-background, and Quit. |
| [settings](settings.md) | Blocked, unimplemented | Settings icon from status/reminder. Schedule edits, interval, login preference, save, cancel, and Back. |
