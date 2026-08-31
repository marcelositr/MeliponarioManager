#!/usr/bin/env python3
"""Exercise the real Tauri desktop WebView through classic W3C WebDriver."""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

DRIVER_URL = "http://127.0.0.1:4444"
W3C_ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"


def request(method: str, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{DRIVER_URL}{path}",
        data=data,
        method=method,
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=5) as response:
        raw = response.read()
    return json.loads(raw.decode("utf-8")) if raw else {}


def wait_for_driver(process: subprocess.Popen[bytes]) -> None:
    deadline = time.monotonic() + 20
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"tauri-driver exited early with status {process.returncode}")
        try:
            request("GET", "/status")
            return
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
            last_error = error
            time.sleep(0.25)
    raise RuntimeError(f"tauri-driver did not become ready: {last_error}")


def create_session(application: Path) -> str:
    payload = {
        "capabilities": {
            "alwaysMatch": {
                "browserName": "wry",
                "tauri:options": {"application": str(application)},
            }
        }
    }
    deadline = time.monotonic() + 20
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            response = request("POST", "/session", payload)
            value = response.get("value", response)
            session_id = value.get("sessionId") or response.get("sessionId")
            if session_id:
                return str(session_id)
            last_error = RuntimeError(f"WebDriver session response had no session id: {response}")
        except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
            last_error = error
        time.sleep(0.5)
    raise RuntimeError(f"could not create Tauri WebDriver session: {last_error}")


def find_element(session_id: str, using: str, value: str) -> str:
    response = request(
        "POST",
        f"/session/{session_id}/element",
        {"using": using, "value": value},
    )
    element = response.get("value", {})
    element_id = element.get(W3C_ELEMENT_KEY) or element.get("ELEMENT")
    if not element_id:
        raise RuntimeError(f"element not found: {using}={value}: {response}")
    return str(element_id)


def element_text(session_id: str, element_id: str) -> str:
    response = request("GET", f"/session/{session_id}/element/{element_id}/text")
    return str(response.get("value", ""))


def wait_for_heading(session_id: str, expected: str) -> None:
    deadline = time.monotonic() + 15
    last_text = ""
    while time.monotonic() < deadline:
        try:
            heading = find_element(session_id, "css selector", "h1")
            last_text = element_text(session_id, heading).strip()
            if last_text == expected:
                return
        except (urllib.error.HTTPError, urllib.error.URLError, RuntimeError):
            pass
        time.sleep(0.25)
    raise AssertionError(f"expected desktop heading {expected!r}, got {last_text!r}")


def stop_process_group(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is not None:
        return
    try:
        os.killpg(process.pid, signal.SIGTERM)
        process.wait(timeout=5)
    except (ProcessLookupError, subprocess.TimeoutExpired):
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} <tauri-application>", file=sys.stderr)
        return 2

    application = Path(sys.argv[1]).resolve()
    if not application.is_file() or not os.access(application, os.X_OK):
        raise RuntimeError(f"Tauri application is not executable: {application}")

    driver = subprocess.Popen(
        ["tauri-driver"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    session_id: str | None = None
    try:
        wait_for_driver(driver)
        session_id = create_session(application)

        wait_for_heading(session_id, "Visão geral")
        agenda_button = find_element(
            session_id,
            "xpath",
            "//button[normalize-space(.)='Abrir Agenda']",
        )
        request("POST", f"/session/{session_id}/element/{agenda_button}/click", {})
        wait_for_heading(session_id, "Agenda")

        print("Desktop WebDriver smoke passed: Visão geral -> Agenda", flush=True)
        return 0
    finally:
        if session_id:
            try:
                request("DELETE", f"/session/{session_id}")
            except Exception:
                pass
        stop_process_group(driver)


if __name__ == "__main__":
    raise SystemExit(main())
