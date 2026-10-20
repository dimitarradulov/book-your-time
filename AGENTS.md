# Working on BYT

BYT (Book Your Time) is a local desktop app for Windows and macOS that reminds people to submit working hours in an existing timesheet system. Work within the agreed first-version scope in [SPEC.md](SPEC.md).

## Commands

Run from the project root. Prerequisites and platform setup are in [README.md](README.md#prerequisites); npm scripts are authoritative in [package.json](package.json).

| Task | Command |
| --- | --- |
| Install locked JavaScript dependencies | `npm ci` |
| Start the native desktop app | `npm run tauri dev` |
| Start only the browser frontend | `npm run dev` |
| Lint frontend code | `npm run lint` |
| Generate Next.js types and check TypeScript | `npm run typecheck` |
| Build the static frontend export | `npm run build` |
| Check Rust compilation | `cargo check --manifest-path src-tauri/Cargo.toml` |
| Check Rust formatting | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` |
| Build the native app without installers | `npm run tauri build -- --no-bundle` |

Tauri starts its own Next.js development server on port 3000. Use one launch path at a time; preserve existing user processes. For isolated native verification, use the verify skill below.

## Read for the task

- **Behavior:** Read [SPEC.md](SPEC.md) before implementing or changing setup, scheduling, reminders, completion, or background behavior. It owns acceptance criteria; mocks contain illustrative dates and URLs.
- **UI:** Read [PRODUCT.md](PRODUCT.md) and [the mock selection](.impeccable/mocks/README.md) before interface work. Variant B is approved. Implementation remains on hold until the user requests it.
- **Native integration:** Read [the Tauri skill](.agents/skills/tauri-v2/SKILL.md) before changing Rust commands, IPC, plugins, capabilities, or packaging.
- **React:** Read the applicable rules in [the React skill](.agents/skills/vercel-react-best-practices/SKILL.md) when writing or refactoring components. Apply rules compatible with static export.
- **Verification:** Follow [the verify skill](.agents/skills/verify/SKILL.md) for UI or native behavior changes. It owns launch isolation, feature journeys, evidence, and cleanup.
- **Icons:** Follow [the icon workflow](README.md#app-icon) when changing `app-icon.svg`; regenerate platform assets through the Tauri CLI.

## Architecture and code

The stack is Tauri 2, Next.js 16 App Router, React 19, TypeScript 5, and Rust (2024 edition). Exact versions live in the package manifests and lockfiles.

- `src/app/` owns pages, layout, and CSS. Match the typed component style in [layout.tsx](src/app/layout.tsx) and visual tokens in [globals.css](src/app/globals.css).
- `src-tauri/src/` owns native application behavior. Keep scheduling and durable reminder state in this layer so they can operate while the window is closed.
- `src-tauri/capabilities/` owns window permissions. Grant only permissions required by the feature being implemented.
- [next.config.ts](next.config.ts) exports static files into `out/`; [tauri.conf.json](src-tauri/tauri.conf.json) embeds them. There is no Node.js server in the packaged app. Server Actions and request-dependent API handlers cannot run there.
- Put interactive UI in Client Components. Access browser and Tauri APIs only at runtime on the client, including avoiding access during build-time prerendering.

Settings and reminder state stay local. Accounts, a backend, synchronization, and browsable completion history are outside v1 scope.

## Validation

For frontend changes, run lint, typecheck, and the static export build. For Rust changes, run the Rust compilation and formatting checks above. For integration or packaging changes, also build the native app. Documentation-only changes need link, command, and factual checks against their sources.

No automated test suite is configured yet. The native verification feature map distinguishes runnable journeys from unimplemented ones. A passing starter screen does not prove reminders, persistence, or tray behavior. Exercise affected user paths through the verify skill and report the platform, evidence, and any blocked checks. macOS verification does not establish Windows behavior.

## Boundaries and git

- Keep changes within the requested task. Changes to the agreed product scope need explicit user direction.
- Edit sources; regenerate build output and generated schemas with their owning tools. Keep secrets and local settings out of commits.
- Preserve unrelated work. When Git is available, inspect the working tree before editing and review the final diff. This checkout currently has no `.git` directory; initialize or connect it only when requested.
- Close the task with the changed behavior or artifact, checks run, and remaining verification limits.

<!-- BEGIN:nextjs-agent-rules -->

## This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->
