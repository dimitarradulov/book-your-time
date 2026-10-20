---
name: verify
description: Verify BYT's native desktop UI after changes, capture user actions and results, and check setup, reminders, status, settings, and background behavior against the feature map.
---

# Verify BYT

Read [the feature index](features/README.md), then the files for the requested journeys. Read `README.md`, `SPEC.md`, and `PRODUCT.md` at the repository root when behavior is disputed. The chosen design is `.impeccable/mocks/variant-b.png`, the compact variant. Its dates and URL are illustrative.

BYT currently has a starter page and a native window. Setup, reminders, status, settings, persistence, tray behavior, and autostart are unimplemented. Report those journeys as blocked with the missing control or precondition. Never substitute a starter-screen pass for a product-feature pass.

## Launch

Use the native macOS app as the primary target. Browser-only checks cannot prove Tauri IPC, default-browser opening, login registration, tray operation, or native persistence. The helper requires Python 3, npm dependencies, Rust, and Xcode Command Line Tools. Install dependencies with `npm ci` if `node_modules` is absent. No account, database, environment secrets, migrations, or seeds exist in the current scaffold.

Run from the repository root, using a new directory for each attempt:

```sh
VERIFY_RUN="$PWD/.audit/verify/$(date -u +%Y%m%dT%H%M%SZ)"
.agents/skills/verify/scripts/control.py launch "$VERIFY_RUN"
```

The helper runs the documented Tauri build command with `--debug --bundles app --ci` and a disposable config override. It builds the real static export, assigns a unique `com.bookyourtime.verify.v...` bundle identifier, copies the app into the run's scratch directory, and launches its executable. `build.log`, `native.log`, `build-command.json`, and `instance.json` identify what ran. No dev server or port is needed. The user's port 3000 and existing BYT instance stay untouched.

The identifier isolates identifier-based Tauri storage. There is currently no product data to seed or clear. Before driving future persistence code, verify that it uses the identifier or an explicit disposable path. Hardcoded production paths or global login registration invalidate this isolation.

One verification run owns `.audit/verify/.lock`. Concurrent verification builds are refused because Next.js output and Cargo's bundle destination are shared. Avoid concurrent source edits or other builds. An already running development server can remain open. Never remove another run's lock or drive an existing user instance.

Ready means the helper's doctor passes **and** the owned native window visibly shows the expected page. For the current scaffold, require the heading `A little nudge for your timesheet.` and the note beginning `Reminders aren’t set up yet.` in its accessibility tree and screenshot. A live PID alone is insufficient. On every failure, run Cleanup before retrying.

## Doctor

Run this read-only check first whenever the instance looks wrong:

```sh
.agents/skills/verify/scripts/control.py doctor "$VERIFY_RUN"
```

It checks the recorded PID and process start identity, executable path and SHA-256, bundle identifier, and source hashes against the built instance. Save its output to `doctor.json` for evidence. It does not prove rendering or UI readiness. Read `native.log` and capture the native window next. A source change requires cleanup and a fresh build, not reusing stale proof.

## Drive

On Codex for macOS, load the installed `computer-use:computer-use` skill and use its `node_repl` + `@oai/sky` API. All UI actions go through that API. Read `instance.json` to obtain this run's exact app path. Use a fresh accessibility tree to find controls by their visible labels, then act on the returned `element_index`. Use screenshot coordinates only when accessibility exposes no usable handle, and record that limitation.

Bootstrap in `node_repl` from the repository root. The lock records the current owned run's absolute path:

```js
var fs = await import('node:fs/promises');
var {fileURLToPath} = await import('node:url');
var sky = (await import('@oai/sky')).sky;
var run = (await fs.readFile(process.cwd() + '/.audit/verify/.lock/owner', 'utf8')).trim();
var instance = JSON.parse(await fs.readFile(run + '/instance.json', 'utf8'));
var app = instance.app;
var state = await sky.get_app_state({app, disableDiff: true});
nodeRepl.write(state.text);
```

`process.cwd()` must be the repository root. Read the actual run path from the shell if the REPL started elsewhere. Use the bundle identifier from `instance.json` if path targeting fails. Do not fall back to the production app identifier. When reusing REPL bindings, assign to the existing variable instead of redeclaring it later in a cell that already reads it.

Capture each state in the REPL:

```js
await fs.writeFile(run + '/before.ax.txt', state.text);
if (!state.screenshot) throw new Error('Native screenshot unavailable');
await fs.copyFile(fileURLToPath(state.screenshot.url), run + '/before.png');
await nodeRepl.emitImage({bytes: await fs.readFile(run + '/before.png'), mimeType: 'image/png'});
```

For an action, append its timestamp, feature ID, entry point, and actual label to `actions.jsonl`. Call `sky.click({app, element_index})` or the corresponding documented API using the index from the tree just read. Fetch the resulting full state with `sky.get_app_state({app, disableDiff: true})`, then save `after.ax.txt` and `after.png` by the same recipe. Follow the matching feature file for assertions and additional evidence. The scaffold has no in-page buttons; native window actions are its only interactive path.

The project has no Playwright, Cypress, WebDriver, CDP, CLI, or HTTP service driver. Tauri uses WKWebView on macOS. Do not assume Electron's debugging port or invent routes, test endpoints, ARIA labels, IPC commands, or persisted file names. Target controls from the running app. Intended labels in blocked feature files are acceptance criteria until implemented.

Windows is a supported product target, but this helper and Computer Use recipe cover macOS only. A real Windows build, WebView2 UI driver, tray, login behavior, installer, signing, and notarization remain unverified. Report that limit with any cross-platform claim.

## Evidence

Keep proof in the absolute `.audit/verify/<run>/` directory printed by launch. Record the feature ID, each entry point attempted, actual actions, assertions, failure reasons, and platform in `result.json` or `result.md`. Capture the action and resulting state together. Include screenshots and full accessibility trees before and after, doctor output, build/runtime logs, and the source/executable hashes in `instance.json`. Visually inspect screenshots for blank content, clipping, or unexpected dialogs.

Exercise real user paths. Internal setters, direct IPC invocations that bypass UI, and injected application state do not prove a journey. Read-only storage inspection can corroborate visible results. For a mutation, record the actual persisted value and restart behavior alongside the screen. The current scaffold has no settings or reminder-state file, so no persistence claim is possible.

Opening a timesheet must produce an observed default-browser navigation. Use a disposable local HTTP destination when a production boundary supports it; capture its requests and resulting browser state. Never use a colleague's real timesheet submission to prove BYT. The five-second gate measures successful URL handoff, not actual submission. Fault simulations belong at an existing external boundary, and their scope must be reported.

No dry-run or test mode exists. Verify any future dry-run by inspecting its actual file, network, browser, and login side effects. Do not infer safety from a mode's name. The mock's two October 2026 dates are not seed data or expected runtime dates. Do not change the host clock or timezone to force scheduling tests. Until disposable time control exists, unmet scheduling preconditions stay blocked.

## Cleanup

```sh
.agents/skills/verify/scripts/control.py cleanup "$VERIFY_RUN"
test -s "$VERIFY_RUN/before.png"
test -s "$VERIFY_RUN/after.png"
test -s "$VERIFY_RUN/actions.jsonl"
```

Run cleanup after success and after every failed attempt. It verifies PID identity before signaling only the native process it started, removes the copied scratch app, releases this run's lock, and marks `instance.json` cleaned. It is safe to repeat. Evidence survives. Never kill by process name or stop the user's development server.

If launch was interrupted while the build is still running, interrupt that launch's terminal session and wait for its build process to exit before cleanup. The helper does not terminate an in-progress compiler. If a future run creates identifier-based settings or login entries, record and remove only those disposable artifacts after proving ownership. This scaffold creates none.

## Helpers

The executable [scripts/control.py](scripts/control.py) supports `launch`, `doctor`, and `cleanup`, each followed by the run directory. It uses Python's standard library and the repository's npm/Tauri commands. It does not automate UI actions. Keep screenshots and action transcripts even when a journey fails.

Use `/maintain-verification-skill` to update this skill and its feature map as BYT changes.
