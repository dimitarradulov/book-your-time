# BYT (Book Your Time)

<!-- impeccable:product-schema 1 -->

## Platform

web

The interface runs inside a Tauri desktop app on Windows and macOS.

## Users

The project owner and a small group of colleagues who submit working hours in an existing company timesheet system.

## Product Purpose

Help people remember to submit hours. Reminders persist until the user explicitly confirms submission; unconfirmed scheduled dates carry into the next reminder.

## Operating Context

The app runs locally in the system tray or menu bar. Opening the timesheet hands its configured URL to the default browser. Scheduling and reminder state work offline using the computer's local time.

## Capabilities and Constraints

- SPEC.md is the agreed first-version product specification.
- One compact window contains initial setup, status/reminder, and settings views. The status view shows the next reminder; the reminder view shows outstanding dates and the required actions. A small settings button opens settings. Confirmed by the user on 2026-10-20.
- Setup collects a timesheet URL and one weekly or monthly schedule. Weekly rules support selected weekdays; monthly rules include the last day of the month.
- Reminder actions are Open timesheet, I've submitted my hours, and snooze for 15 minutes, 30 minutes, or one hour.
- Completion remains disabled until the URL successfully opens and at least five seconds pass. Confirmation completes all dates shown.
- Repeat interval defaults to five minutes and is configurable. Start at login defaults on and can be disabled.
- Automatic booking reminders play a brief double chime, including focused repeats, snooze expiry, and same-day catch-up. Reminder sound defaults on, can be disabled in settings, and respects system volume and mute. Manual navigation and missing-date prompts stay silent. No preview or sound customization is included.
- Settings and reminder state persist locally. No accounts, backend, synchronization, manager dashboard, or browsable completion history in v1.
- Current implementation is a scaffold. Mockups demonstrate proposed interface states; they do not indicate implemented scheduling or native behavior.

## Brand Commitments

The interface must be very simple and professional, containing only elements needed for the goal. The existing scaffold uses system typography, muted green, warm white, and dark text.

## Evidence on Hand

SPEC.md and README.md describe the agreed scope. The existing page, stylesheet, and editable app icon supply the initial visual cues. Mockup dates and URLs are illustrative.

## Product Principles

- Keep submitting hours as the central task.
- Show unconfirmed dates clearly without asserting that company timesheets are overdue.
- Distinguish opening the timesheet from confirming submission.
- Keep setup and settings compact and background operation quiet.
