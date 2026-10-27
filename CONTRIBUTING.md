# Contributing to BYT

BYT is a local desktop app for macOS and Windows. Read [SPEC.md](SPEC.md) before
changing product behavior. It owns the agreed first-version scope. Read
[AGENTS.md](AGENTS.md) for repository conventions and task-specific guidance.

## Report a bug or propose a change

[Open an issue](https://github.com/dimitarradulov/book-your-time/issues) with the
BYT version, operating system, steps to reproduce, expected result, and actual
result. Include screenshots or logs when they explain the problem. Remove
credentials, private timesheet URLs, and company information.

Discuss changes to the agreed scope before implementing them. Accounts, a backend,
synchronization, and a browsable completion history are outside the first version.

## Prerequisites

Install these tools before building from source:

| Tool | Requirement |
| --- | --- |
| Node.js and npm | Node.js 22 is used in CI. The declared minimum is 20.9.0. |
| Rust and Cargo | Stable Rust through [rustup](https://rustup.rs/). The crate requires Rust 1.90 or newer. |
| macOS native tools | Xcode Command Line Tools, installed with `xcode-select --install`, or full Xcode. |
| Windows native tools | Microsoft C++ Build Tools with **Desktop development with C++**, plus Microsoft Edge WebView2. |

Follow the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/)
for installation details. Restart your terminal after installing Rust so `cargo`
is available on your `PATH`.

## Start development

Fork the repository if you do not have write access. Use your fork's clone URL
below, or clone upstream if you have write access. Create a branch for the change:

```sh
git clone https://github.com/dimitarradulov/book-your-time.git
cd book-your-time
git switch -c your-change
npm ci
npm run tauri dev
```

Tauri starts the Next.js server at `http://127.0.0.1:3000` and opens the native
window. Use one launch path at a time. Keep port 3000 available and preserve
other developers' running processes.

For frontend work in a browser, use `npm run dev` instead. Browser previews do not
exercise native commands, persistence, reminders, the tray, or launch at login.
Local development and checks do not require signing credentials or GitHub secrets.

## Find the code

| Location | Responsibility |
| --- | --- |
| `src/app/` | Pages, layout, styles, and interactive React components. |
| `src-tauri/src/lib.rs` | Tauri setup, native commands, plugins, and tray menu. |
| `src-tauri/src/preferences.rs` | Preference validation and local storage. |
| `src-tauri/src/reminders.rs` | Reminder state, schedules, snooze, and completion rules. |
| `src-tauri/src/reminder_runtime.rs` | Native scheduling, window behavior, and browser handoff. |
| `src-tauri/src/reminder_sound.rs` | Reminder sound playback. |
| `src-tauri/capabilities/` | Native permissions granted to the desktop window. |
| `src-tauri/tauri.conf.json` | Window settings, frontend build commands, and packaging. |
| `next.config.ts` | Static frontend export into `out/`. |
| `.github/workflows/release-desktop.yml` | Installer builds and draft GitHub Releases. |

The packaged app embeds static frontend files. It has no Node.js server, so
Server Actions and request-dependent API handlers cannot run there. Keep
interactive UI in Client Components. Access browser and Tauri APIs at runtime,
not during build-time prerendering.

Keep scheduling and durable reminder state in Rust so they continue while the
window is closed. Grant only permissions required by the feature.

Preferences live in `preferences.json` in Tauri's app configuration directory.
Reminder state lives in `reminders.json` beside it. The directory depends on the
platform and bundle identifier. Do not commit local settings or generated data.

For interface changes, read [PRODUCT.md](PRODUCT.md) and
[the approved mock selection](.impeccable/mocks/README.md). Use the project-local
[React](.agents/skills/vercel-react-best-practices/SKILL.md) and
[Tauri](.agents/skills/tauri-v2/SKILL.md) guidance where applicable. Read relevant
Next.js guides in `node_modules/next/dist/docs/` before changing Next.js code.

## Check your change

Run commands from the repository root. `package.json` owns the npm scripts.

| Check | Command |
| --- | --- |
| Frontend lint | `npm run lint` |
| Next.js types and TypeScript | `npm run typecheck` |
| Static frontend export | `npm run build` |
| Rust compilation | `cargo check --manifest-path src-tauri/Cargo.toml` |
| Rust formatting | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` |
| Native behavior and persistence tests | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Native build without installers | `npm run tauri build -- --no-bundle` |

For frontend changes, run lint, typecheck, and the static export build. For Rust
changes, run compilation, formatting, and the affected behavior tests. For
integration or packaging changes, also build the native app. For documentation
changes, check links, commands, and claims against their sources.

### Verify native behavior

Exercise affected user paths in the native app. Check setup, saved settings,
reminders, snooze, browser opening, confirmation, tray access, and launch at login
as applicable. Confirm persistence by quitting and reopening the app.

Follow the [native verification skill](.agents/skills/verify/SKILL.md) for isolated
macOS verification, screenshots, evidence, and cleanup. Its feature map still
contains scaffold-era status descriptions. Inspect the running app and record
actual controls and results rather than treating that status as current coverage.

Use a disposable instance for checks that change settings or schedules. The
helper assigns a unique bundle identifier and records process ownership. Do not
drive an existing user instance, change the system clock, or submit real company
hours to test a reminder.

Report the platform and any blocked checks. A macOS pass does not establish
Windows behavior. A successful Windows build does not prove installation,
WebView2 rendering, tray behavior, or login startup. Those require Windows checks.

## Build installers

Build on macOS for Mac packages and on Windows for Windows installers:

```sh
npm run tauri build
```

Tauri runs `npm run build`, embeds the static export, compiles Rust, and writes
packages under `src-tauri/target/release/bundle/`. Builds with an explicit
`--target` use `src-tauri/target/<target>/release/bundle/` instead.

To build only the Windows setup installer, use:

```sh
npm run tauri -- build --bundles nsis
```

Local builds do not require signing. To distribute signed Mac builds, supply the
signing and notarization environment described below.

## App icon

Edit [app-icon.svg](app-icon.svg), then regenerate platform assets with Tauri:

```sh
npm run tauri -- icon ./app-icon.svg
npm run tauri -- build
```

Generated icons live in `src-tauri/icons/` and are referenced by
`src-tauri/tauri.conf.json`. Edit the SVG source and regenerate the assets rather
than editing generated files. Reinstall and reopen the app to inspect its icon.

## Prepare a release

The [Release desktop workflow](.github/workflows/release-desktop.yml) builds
signed and notarized Mac DMGs for Apple silicon and Intel, plus an unsigned
Windows x64 NSIS installer. Only its final job writes the draft release, after
all three installers pass checks. It never publishes automatically.

The workflow verifies the Mac app inside each finished DMG, including its signing
identity, team, notarization ticket, and Gatekeeper acceptance. It verifies that
both the Windows application and installer are unsigned. Windows signing and
automatic app updates are not configured.

Repository maintainers configure these GitHub Actions secrets for Mac signing:

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` containing the Developer ID Application certificate and private key. |
| `APPLE_CERTIFICATE_PASSWORD` | Password protecting the exported `.p12`. |
| `APPLE_SIGNING_IDENTITY` | Full `Developer ID Application: Name (TEAMID)` certificate name. |
| `APPLE_ID` | Apple Account email. |
| `APPLE_PASSWORD` | Apple app-specific password for notarization. |
| `APPLE_TEAM_ID` | Apple Developer Team ID. |

Keep all credential values out of the repository. Windows build jobs receive no
Apple secrets. See the [Tauri signing guide](https://v2.tauri.app/distribute/sign/macos/)
for certificate and notarization setup.

To prepare a versioned release:

1. Set the same version in `package.json`, `package-lock.json`,
   `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Commit the changes and push a matching `v<version>` tag on the commit to release.
3. Wait for the workflow to finish, then open its draft in GitHub Releases.
4. Install the downloads on the target platforms and check the affected user paths.
5. Review the notes and publish when the release is ready.

Use a unique `desktop-test-*` tag for a test draft, without consuming a version
tag. You can also select **Actions → Release desktop → Run workflow** after the
workflow is on the default branch. Manual runs derive the tag from the app version.
Existing tags and drafts must point to the same commit; the workflow refuses to
replace a published release.

## Open a pull request

Keep changes focused and preserve unrelated work. Include the problem, the
resulting behavior, and the checks you ran. Mention unverified platforms and any
remaining limitations. Add screenshots for interface changes and describe the
user path they show.

When updating README screenshots, capture the native app with example settings.
Keep private URLs and company information out of the images. Label the platform
and use the isolated verification process above.
