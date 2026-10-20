#!/usr/bin/env python3
"""Build, identify, and stop a disposable macOS BYT verification instance."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
RUNS = ROOT / ".audit/verify"
LOCK = RUNS / ".lock"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sources():
    files = [ROOT / name for name in (
        "package.json", "package-lock.json", "next.config.ts", "tsconfig.json",
        "src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src-tauri/build.rs",
        "src-tauri/tauri.conf.json",
    )]
    for folder in ("src", "src-tauri/src", "src-tauri/capabilities"):
        files.extend(p for p in (ROOT / folder).rglob("*") if p.is_file())
    return {str(p.relative_to(ROOT)): digest(p) for p in sorted(files)}


def process(pid):
    result = subprocess.run(
        ["ps", "-p", str(pid), "-o", "lstart=", "-o", "command="],
        capture_output=True, text=True, check=False,
    )
    return result.stdout.strip() if result.returncode == 0 else None


def save(run, state):
    (run / "instance.json").write_text(json.dumps(state, indent=2) + "\n")


def launch(run):
    if sys.platform != "darwin":
        raise RuntimeError("This native helper is macOS-only. Windows needs its own UI driver.")
    if run.exists():
        raise RuntimeError("Use a new run directory. Existing evidence must survive.")
    RUNS.mkdir(parents=True, exist_ok=True)
    LOCK.mkdir()
    (LOCK / "owner").write_text(str(run))
    run.mkdir()
    identity = "com.bookyourtime.verify.v" + hashlib.sha256(str(run).encode()).hexdigest()[:16]
    state = {"status": "building", "root": str(ROOT), "run": str(run),
             "identifier": identity, "sources": sources()}
    save(run, state)
    config = run / "tauri.verify.json"
    config.write_text(json.dumps({"productName": "BYT Verify", "identifier": identity}))
    command = ["npm", "run", "tauri", "--", "build", "--debug", "--bundles", "app",
               "--config", str(config), "--ci"]
    (run / "build-command.json").write_text(json.dumps(command, indent=2) + "\n")
    print(f"Building. Log: {run / 'build.log'}", flush=True)
    with (run / "build.log").open("w") as log:
        subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
    if state["sources"] != sources():
        raise RuntimeError("Source changed during build. Clean up and launch a fresh run.")
    app = run / "scratch/BYT Verify.app"
    shutil.copytree(ROOT / "src-tauri/target/debug/bundle/macos/BYT Verify.app", app)
    executable = app / "Contents/MacOS/byt"
    state.update(app=str(app), executable=str(executable), sha256=digest(executable))
    with (run / "native.log").open("w") as log:
        child = subprocess.Popen([str(executable)], cwd=ROOT, stdout=log,
                                 stderr=subprocess.STDOUT, start_new_session=True)
    state.update(pid=child.pid, process=process(child.pid), status="running")
    save(run, state)
    time.sleep(2)
    if child.poll() is not None:
        raise RuntimeError(f"BYT exited with {child.returncode}. Read native.log.")
    doctor(run)


def doctor(run):
    state = json.loads((run / "instance.json").read_text())
    if state["status"] != "running":
        raise RuntimeError(f"Instance status is {state['status']}.")
    if process(state["pid"]) != state["process"] or not state["process"]:
        raise RuntimeError("Owned native process is gone or its PID was reused.")
    if state["executable"] not in state["process"]:
        raise RuntimeError("PID does not belong to this run's executable.")
    if digest(Path(state["executable"])) != state["sha256"] or sources() != state["sources"]:
        raise RuntimeError("Build or source changed. Launch a fresh run.")
    with (Path(state["app"]) / "Contents/Info.plist").open("rb") as file:
        info = plistlib.load(file)
    if info["CFBundleIdentifier"] != state["identifier"]:
        raise RuntimeError("Bundle identifier does not match this run.")
    print(json.dumps({"doctor": "PASS", "pid": state["pid"], "app": state["app"],
                      "identifier": state["identifier"], "sha256": state["sha256"],
                      "ui_readiness": "Require the native accessibility tree and screenshot next."}, indent=2))


def cleanup(run):
    manifest = run / "instance.json"
    state = json.loads(manifest.read_text()) if manifest.exists() else None
    if state and state.get("pid"):
        current = process(state["pid"])
        if current and current != state["process"]:
            raise RuntimeError("PID identity changed. Refusing to signal another process.")
        if current:
            os.kill(state["pid"], signal.SIGTERM)
            for _ in range(50):
                if process(state["pid"]) is None:
                    break
                time.sleep(0.1)
            else:
                raise RuntimeError("Owned process has not stopped. Keep the lock and investigate.")
    shutil.rmtree(run / "scratch", ignore_errors=True)
    if LOCK.exists() and (LOCK / "owner").read_text() == str(run):
        shutil.rmtree(LOCK)
    if state:
        state["status"] = "cleaned"
        save(run, state)
    print(f"Cleaned owned instance. Evidence retained at {run}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("launch", "doctor", "cleanup"))
    parser.add_argument("run", type=Path, help="A new direct child of .audit/verify")
    args = parser.parse_args()
    run_path = args.run.resolve()
    if run_path.parent != RUNS or run_path.name.startswith("."):
        parser.error("Run directory must be a non-hidden direct child of .audit/verify.")
    try:
        {"launch": launch, "doctor": doctor, "cleanup": cleanup}[args.action](run_path)
    except Exception as error:
        print(str(error), file=sys.stderr)
        if args.action == "launch":
            print(f"Run cleanup even after failure: {Path(__file__).relative_to(ROOT)} cleanup {run_path}", file=sys.stderr)
        sys.exit(1)
