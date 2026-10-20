# Native starter screen

Opening BYT currently displays its purpose and explains that reminders are not configured. This is the only implemented app view.

## Sub-features

- `scaffold.launch` opens the actual native window with the exported frontend.
- `scaffold.content` shows the BYT identity, timesheet purpose, local-data message, and setup note.
- `scaffold.zoom` keeps the starter content visible when the user changes the native window size.

## How to get to it (user POV)

- Launch the desktop app. There are no other in-page entry points or controls yet.
- Use the native green window button's zoom action to change the window size.

## Driving it with Computer Use

Preconditions: Launch an isolated native instance and pass Doctor using the parent skill. Use its `app`, `run`, `sky`, and `fs` REPL bindings.

- Open. Run `var state = await sky.get_app_state({app, disableDiff: true}); nodeRepl.write(state.text);`. Require a window titled `BYT`, HTML content titled `BYT`, and URL `tauri://localhost/`. This proves embedded native content rather than a browser page or live Next.js server.
- Read. Require the heading `A little nudge for your timesheet.`, text `Your settings and reminders stay on your computer.`, and the setup note beginning `Reminders aren’t set up yet.`. Save `before.ax.txt` and `before.png` using the parent skill's recipe. Inspect the screenshot.
- Zoom. Find the full screen button with secondary action `zoom the window` in the fresh tree. Record the action in `actions.jsonl`, then call `await sky.perform_secondary_action({app, element_index: zoomIndex, action: 'zoom the window'});` with that observed index. `zoomIndex` comes from the current tree, not a fixed constant. Fetch a fresh full state and save `after.ax.txt` and `after.png`.
- Assert. Require the same heading and setup note after zoom. Inspect both screenshots and verify that the window size changed and text remains readable without clipping. Record the action result and assertions in `result.json`. A size change without readable content fails.
- Finish. Run Cleanup and require the screenshots, action transcript, and result to survive. Confirm the owned PID is gone. If other BYT processes existed before launch, verify their original PIDs remain alive.

## Gotchas

- The window title stays `BYT`; the verification bundle is named `BYT Verify` and has a unique identifier. Target the run's exact app path.
- The current accessibility tree exposes a native `full screen button` and its `zoom the window` secondary action. A normal click can enter full screen instead. Re-read the tree before acting.
- There are no setup fields, settings icon, or reminder buttons in this page. Their absence blocks product journeys; it does not fail this scaffold-specific check.
- Zoom proves native layout behavior. It does not prove scheduling, storage, IPC, or tray operation.
