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
NATIVE_DRIVER_URL = "http://127.0.0.1:4445"
W3C_ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf"


def request_at(
    base_url: str,
    method: str,
    path: str,
    payload: dict[str, Any] | None = None,
    timeout: float = 5,
) -> dict[str, Any]:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        f"{base_url}{path}",
        data=data,
        method=method,
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=timeout) as response:
        raw = response.read()
    return json.loads(raw.decode("utf-8")) if raw else {}


def request(
    method: str,
    path: str,
    payload: dict[str, Any] | None = None,
    *,
    timeout: float = 5,
) -> dict[str, Any]:
    return request_at(DRIVER_URL, method, path, payload, timeout=timeout)


def wait_for_endpoint(
    process: subprocess.Popen[bytes],
    base_url: str,
    label: str,
    timeout: float,
) -> None:
    deadline = time.monotonic() + timeout
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"tauri-driver exited early with status {process.returncode}")
        try:
            request_at(base_url, "GET", "/status", timeout=1)
            print(f"{label} ready at {base_url}", flush=True)
            return
        except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
            last_error = error
            time.sleep(0.25)
    raise RuntimeError(f"{label} did not become ready at {base_url}: {last_error}")


def wait_for_drivers(process: subprocess.Popen[bytes]) -> None:
    wait_for_endpoint(process, DRIVER_URL, "tauri-driver", 20)
    # tauri-driver can accept connections before the native WebKitWebDriver
    # remote end is actually ready. Wait for the native port as well before
    # attempting POST /session.
    wait_for_endpoint(process, NATIVE_DRIVER_URL, "WebKitWebDriver", 30)


def create_session(application: Path) -> str:
    payload = {
        "capabilities": {
            "alwaysMatch": {
                "browserName": "wry",
                "tauri:options": {
                    "application": str(application),
                    "args": [],
                    "webviewOptions": {},
                },
            }
        }
    }
    print("Creating Tauri WebDriver session...", flush=True)
    try:
        response = request("POST", "/session", payload, timeout=60)
    except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError, json.JSONDecodeError) as error:
        raise RuntimeError(f"could not create Tauri WebDriver session: {error}") from error

    value = response.get("value", response)
    session_id = value.get("sessionId") or response.get("sessionId")
    if not session_id:
        raise RuntimeError(f"WebDriver session response had no session id: {response}")

    print(f"Tauri WebDriver session created: {session_id}", flush=True)
    return str(session_id)


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


def click_element(session_id: str, element_id: str) -> None:
    request(
        "POST",
        f"/session/{session_id}/execute/sync",
        {
            "script": "arguments[0].click();",
            "args": [{W3C_ELEMENT_KEY: element_id}],
        },
    )


def wait_for_heading(session_id: str, expected: str) -> None:
    deadline = time.monotonic() + 15
    last_text = ""
    while time.monotonic() < deadline:
        try:
            heading = find_element(session_id, "css selector", "h1")
            last_text = element_text(session_id, heading).strip()
            if last_text == expected:
                print(f"Desktop heading reached: {expected}", flush=True)
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


def print_driver_log(log_path: Path) -> None:
    try:
        output = log_path.read_text(encoding="utf-8", errors="replace").strip()
    except OSError as error:
        print(f"Could not read tauri-driver log: {error}", file=sys.stderr)
        return
    if output:
        print("\n--- tauri-driver / WebKitWebDriver log ---", file=sys.stderr)
        print(output, file=sys.stderr)
        print("--- end driver log ---", file=sys.stderr)


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} <tauri-application>", file=sys.stderr)
        return 2

    application = Path(sys.argv[1]).resolve()
    if not application.is_file() or not os.access(application, os.X_OK):
        raise RuntimeError(f"Tauri application is not executable: {application}")

    log_root = Path(os.environ.get("RUNNER_TEMP", "/tmp"))
    driver_log_path = log_root / "tauri-driver-webdriver.log"
    session_id: str | None = None

    with driver_log_path.open("wb") as driver_log:
        driver = subprocess.Popen(
            ["tauri-driver", "--port", "4444", "--native-port", "4445"],
            stdout=driver_log,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            wait_for_drivers(driver)
            session_id = create_session(application)

            wait_for_heading(session_id, "Visão geral")
            agenda_button = find_element(
                session_id,
                "xpath",
                "//button[normalize-space(.)='Abrir Agenda']",
            )
            click_element(session_id, agenda_button)
            wait_for_heading(session_id, "Agenda")

            print("Desktop WebDriver smoke passed: Visão geral -> Agenda", flush=True)
            return 0
        except Exception:
            stop_process_group(driver)
            driver_log.flush()
            print_driver_log(driver_log_path)
            raise
        finally:
            if session_id:
                try:
                    request("DELETE", f"/session/{session_id}")
                except Exception:
                    pass
            stop_process_group(driver)


if __name__ == "__main__":
    raise SystemExit(main())
