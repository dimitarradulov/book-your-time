# BYT

BYT (Book Your Time) is a desktop app that persistently reminds you to submit your working hours in your
company's timesheet system.

Stack: **Tauri 2 + Next.js (React and TypeScript)**, targeting **Windows and macOS**. Settings and
reminder state stay on your computer; no account or backend is required.

Setup and Settings save the timesheet URL, weekly schedule or monthly booking dates, local time,
repeat interval, and start-at-login preference on this computer. Saved options
restore after quitting and reopening; Cancel and Back discard unsaved edits.
Start at login defaults on and is applied when saving. Settings are stored in
`preferences.json` in Tauri’s app configuration directory.
Reminders run in the native app, including while its window is closed. Due and
missed dates persist in `reminders.json` beside settings; older unconfirmed dates
join the next scheduled reminder. Snooze for 15 minutes, 30 minutes, or one hour.
Opening the timesheet uses the default browser; confirming submission is a
separate action available five seconds after a successful browser handoff.
Confirmation clears the listed dates without submitting anything to the website.
The tray/menu bar provides pending reminders, settings, and Quit BYT.

Automatic booking reminders play a brief double chime, including repeats while
the window is focused, snooze expiry, and same-day missed reminders after startup
or wake. Sound defaults on. Turn off **Reminder sound** in Settings to silence it.
Sound respects system volume and mute. Opening BYT or manually viewing pending
reminders stays silent, as do prompts for missing booking dates.

Schedules use the computer's current local time. Monthly schedules have one
chosen booking date per month and a shared reminder time. Enter dates as your
company announces them, one month at a time or several months ahead. Future
dates can change; dates whose reminders are already due remain protected.
If next month's date is missing, BYT prompts once seven days before that month
begins, or catches up when it next runs. A visible notice remains until a date
is entered. Months without a date do not create booking reminders, and pending
bookings remain saved for the next configured date.
Automatic login launches run in the background. See [SPEC.md](SPEC.md) for the
agreed first-version behavior.

Run the native behavior and persistence checks with:

```sh
cargo test --manifest-path src-tauri/Cargo.toml
```

Native UI verification follows [.agents/skills/verify/SKILL.md](.agents/skills/verify/SKILL.md).
Its scaffold-era feature map is older than the implementation; record actual
controls and outcomes when exercising reminders. Windows behavior still needs
verification on Windows.

## Prerequisites

- Node.js 20.9 or newer and npm (Node.js 22 LTS recommended).
- Stable Rust installed through [rustup](https://rustup.rs/).
- macOS: Xcode Command Line Tools (`xcode-select --install`), or full Xcode.
- Windows: Microsoft C++ Build Tools with the **Desktop development with C++**
  workload, and Microsoft Edge WebView2.

See the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for
platform-specific setup. After installing Rust, restart your terminal so `cargo`
is available on your PATH.

## Development

```sh
npm ci
npm run tauri dev
```

Tauri starts the Next.js development server at `http://127.0.0.1:3000` and opens
the desktop window. Keep port 3000 available. To work on the frontend in a
browser, run `npm run dev` instead.

```sh
npm run lint
npm run typecheck
cargo check --manifest-path src-tauri/Cargo.toml
```

## Build

```sh
npm run tauri build
```

The build exports Next.js into `out/`, then embeds those static files in the
native app. Installers are written to `src-tauri/target/release/bundle/`. Build on
macOS for macOS packages and on Windows for Windows installers. To sign local
builds, supply your own signing environment. The release workflow supplies
macOS signing credentials through GitHub Actions secrets.

Run `npm run build` to verify only the frontend export, or
`npm run tauri build -- --no-bundle` to compile the desktop app without creating
installers.

## Release desktop installers

[Release desktop](.github/workflows/release-desktop.yml) builds signed and
notarized Apple silicon and Intel DMGs, plus an unsigned Windows x64 setup
installer for testing. The Mac apps must pass signature, notarization ticket,
and Gatekeeper checks. The Windows app and installer must be unsigned.
All three installers must pass before the workflow prepares a draft GitHub
Release. It never publishes the release automatically.

Windows signing is not configured. The test installer may show an
unknown-publisher or SmartScreen warning. Windows jobs receive no Apple secrets.

Add these macOS repository secrets under **Settings → Secrets and variables → Actions**:

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64-encoded Developer ID Application certificate and private key exported as `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Password protecting the exported `.p12` |
| `APPLE_SIGNING_IDENTITY` | Full `Developer ID Application: Name (TEAMID)` certificate name |
| `APPLE_ID` | Apple Account email |
| `APPLE_PASSWORD` | Apple app-specific password for notarization |
| `APPLE_TEAM_ID` | Apple Developer Team ID |

Keep credentials out of this repository. See the
[Tauri macOS signing guide](https://v2.tauri.app/distribute/sign/macos/) for creating
the certificate and notarization credentials.

Set the same version in `package.json`, `package-lock.json`,
`src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. Commit the changes, then
push a matching version tag from the commit to release:

```sh
git tag v0.1.0
git push origin v0.1.0
```

To test the workflow before preparing a versioned release, push a unique
`desktop-test-*` tag, such as `desktop-test-20261027-1`, on the commit to check. This
also works for the first run before the workflow reaches the default branch.
Test tags produce separate draft prereleases and do not consume the version tag.

After the workflow exists on the default branch, you can also select
**Actions → Release desktop → Run workflow** and choose the branch to build.
Manual runs derive the release tag from the app version. Existing tags and
drafts must target that same commit. To release different code, bump the version;
the workflow refuses to replace a published release.

Open the draft in GitHub Releases and download the matching DMG or Windows
`-setup.exe`. Check installation, launch, saved settings, reminders, tray behavior,
and start at login on each platform before publishing. Intel runtime behavior
needs an Intel Mac check even when its build succeeds on an Apple silicon runner.
A Windows build does not prove Windows installation or runtime behavior. Follow
the native verification skill for isolated Mac app checks.

Releases in a private repository require repository access to download. Public
distribution requires a public repository or a separate public distribution
location. Publishing a GitHub Release does not add automatic updates to BYT.

## App icon

The editable icon source is [`app-icon.svg`](app-icon.svg): warm white BYT
lettering on the app's muted green, with transparent padding for the macOS Dock.
The letters are vector paths, so no font installation is needed.

After editing the source, regenerate the platform icons and rebuild:

```sh
npm run tauri -- icon ./app-icon.svg
npm run tauri -- build
```

Generated files live in `src-tauri/icons/` and are already referenced by
`src-tauri/tauri.conf.json`. Install and reopen the rebuilt app to use the icon.

## Project structure

- `src/app/`: Next.js App Router pages, layout, and styles.
- `src-tauri/src/`: Rust desktop entry point and Tauri application setup.
- `src-tauri/tauri.conf.json`: Native window and frontend build configuration.
- `src-tauri/capabilities/`: Permissions granted to the desktop window.

Next.js uses [static export](https://v2.tauri.app/start/frontend/nextjs/) because
the packaged app runs without a Node.js server. Use client components for
interactive UI and call Tauri APIs from the client; runtime server actions and
API routes are unavailable. Local preference persistence and scheduling run in
the Rust desktop layer.
