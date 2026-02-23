#!/usr/bin/env python3
import json
import os
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

if os.environ.get("KROMA_ENABLE_LEGACY_SCRIPTS", "").strip().lower() not in {"1", "true", "yes", "on"}:
    raise SystemExit(
        "Legacy dispatch script disabled. Use the Rust backend/runtime (src-tauri). "
        "Set KROMA_ENABLE_LEGACY_SCRIPTS=1 only for explicit migration fallback."
    )


def dispatch_instruction_http(
    *,
    target_url: str,
    token: str | None,
    payload: dict,
    timeout_sec: float = 20.0,
    retries: int = 2,
    backoff_sec: float = 1.5,
):
    if not target_url:
        return {"ok": False, "error": "missing_target_url", "attempts": 0}

    body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    attempt = 0
    last_error = None
    while attempt <= retries:
        attempt += 1
        req = Request(target_url, data=body, method="POST", headers={"Content-Type": "application/json"})
        if token:
            req.add_header("Authorization", f"Bearer {token}")
        try:
            with urlopen(req, timeout=timeout_sec) as res:
                raw = (res.read() or b"").decode("utf-8")
                parsed = json.loads(raw) if raw else {}
                return {
                    "ok": True,
                    "attempts": attempt,
                    "http_status": int(getattr(res, "status", 200)),
                    "response": parsed,
                }
        except HTTPError as exc:
            content = ""
            try:
                content = (exc.read() or b"").decode("utf-8")
            except Exception:
                content = ""
            last_error = f"http_{exc.code}:{content or str(exc)}"
        except URLError as exc:
            last_error = f"url_error:{exc.reason}"
        except Exception as exc:
            last_error = str(exc)

        if attempt <= retries:
            time.sleep(backoff_sec * attempt)

    return {"ok": False, "attempts": attempt, "error": last_error or "unknown_dispatch_error"}
