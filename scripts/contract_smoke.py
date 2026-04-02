#!/usr/bin/env python3
"""
Backend API Contract Smoke Test

Tests the core API endpoints defined in openapi/backend-api.openapi.yaml
to verify the backend is functioning correctly.

Usage:
    python contract_smoke.py --base-url http://127.0.0.1:8788 --project-slug smoke_test
    python contract_smoke.py --base-url http://127.0.0.1:8788 --api-token YOUR_TOKEN
"""
from __future__ import annotations

import argparse
import json
import urllib.error
import urllib.request


def request_json(base_url: str, method: str, path: str, payload: dict | None = None, api_token: str | None = None) -> dict:
    """Make HTTP request and return JSON response."""
    data = None
    headers = {"Content-Type": "application/json"}
    if api_token:
        headers["Authorization"] = f"Bearer {api_token}"
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(f"{base_url}{path}", data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:  # noqa: S310
            raw = resp.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        raise SystemExit(f"{method} {path} failed: HTTP {exc.code} {body}") from exc
    except urllib.error.URLError as exc:
        raise SystemExit(f"{method} {path} failed: {exc.reason}") from exc
    if not raw:
        return {}
    return json.loads(raw)


def main():
    parser = argparse.ArgumentParser(description="Backend API contract smoke test")
    parser.add_argument("--base-url", default="http://127.0.0.1:8788", help="Backend API base URL")
    parser.add_argument("--project-slug", default="contract_demo", help="Project slug to use")
    parser.add_argument("--api-token", help="API bearer token (optional if KROMA_API_AUTH_DEV_BYPASS=true)")
    args = parser.parse_args()

    base_url = args.base_url.rstrip("/")
    slug = args.project_slug.strip() or "contract_demo"
    api_token = args.api_token

    print("[contract-smoke] Testing backend API contract")
    print(f"[contract-smoke] Base URL: {base_url}")
    print(f"[contract-smoke] Project slug: {slug}")
    if api_token:
        print(f"[contract-smoke] Using API token: {api_token[:8]}...")
    else:
        print("[contract-smoke] No API token provided - requires KROMA_API_AUTH_DEV_BYPASS=true")
    print()

    # Health check (no auth required)
    print("[contract-smoke] 1/8 - Health check")
    health = request_json(base_url, "GET", "/health", api_token=api_token)
    assert health.get("ok") is True, "Health check failed"
    print(f"           ✓ Backend healthy: {health.get('service')} v{health.get('version')}")
    print(f"           ✓ Routes: {health.get('route_count')}")

    # Create project
    print("[contract-smoke] 2/8 - Upsert project")
    project = request_json(base_url, "POST", "/api/projects", {"name": "Contract Demo", "slug": slug}, api_token=api_token)
    assert "project" in project, "Project creation failed"
    print(f"           ✓ Project created: {slug}")

    # Prompt templates CRUD
    print("[contract-smoke] 3/8 - Prompt templates CRUD")
    created_template = request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/prompt-templates",
        {"name": "default-shot", "template_text": "A cinematic shot of {subject}"},
        api_token=api_token,
    )
    assert "prompt_template" in created_template, "Template creation failed"
    template_id = created_template["prompt_template"]["id"]
    print(f"           ✓ Template created: {template_id}")

    templates_list = request_json(base_url, "GET", f"/api/projects/{slug}/prompt-templates", api_token=api_token)
    assert "templates" in templates_list or "prompt_templates" in templates_list, "Template list failed"

    template_detail = request_json(base_url, "GET", f"/api/projects/{slug}/prompt-templates/{template_id}", api_token=api_token)
    assert "prompt_template" in template_detail, "Template detail failed"

    updated = request_json(
        base_url,
        "PUT",
        f"/api/projects/{slug}/prompt-templates/{template_id}",
        {"template_text": "A cinematic close-up of {subject}"},
        api_token=api_token,
    )
    assert updated.get("ok") is True, "Template update failed"

    deleted = request_json(base_url, "DELETE", f"/api/projects/{slug}/prompt-templates/{template_id}", api_token=api_token)
    assert deleted.get("ok") is True, "Template deletion failed"
    print("           ✓ Templates CRUD complete")

    # Chat sessions
    print("[contract-smoke] 4/8 - Chat sessions")
    sess = request_json(base_url, "POST", f"/api/projects/{slug}/chat/sessions", {"title": "Contract session"}, api_token=api_token)
    assert "session" in sess, "Session creation failed"
    session_id = sess["session"]["id"]
    print(f"           ✓ Session created: {session_id}")

    # Add message
    print("[contract-smoke] 5/8 - Add chat message")
    msg = request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/chat/sessions/{session_id}/messages",
        {"role": "user", "content_text": "Create instruction"},
        api_token=api_token,
    )
    assert "message" in msg, "Message creation failed"
    print("           ✓ Message added")

    # Agent instructions
    print("[contract-smoke] 6/8 - Agent instructions")
    ins = request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/agent/instructions",
        {"instruction_type": "pipeline_run", "dispatch_to_agent": False, "payload_json": {"stage": "style", "candidates": 2}},
        api_token=api_token,
    )
    assert "instruction" in ins, "Instruction creation failed"
    instr_id = ins["instruction"]["id"]
    print(f"           ✓ Instruction created: {instr_id}")

    # Events
    print("[contract-smoke] 7/8 - Instruction events")
    events = request_json(base_url, "GET", f"/api/projects/{slug}/agent/instructions/{instr_id}/events", api_token=api_token)
    assert "events" in events or "instruction_events" in events, "Events fetch failed"
    print("           ✓ Events retrieved")

    # Secrets CRUD
    print("[contract-smoke] 8/8 - Project secrets")
    secret_resp = request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/secrets",
        {"provider_code": "openai", "secret_name": "api_key", "secret_value": "sk-test-contract-123456"},
        api_token=api_token,
    )
    assert secret_resp.get("ok") is True, "Secret creation failed"
    print("           ✓ Secret created")

    secrets_list = request_json(base_url, "GET", f"/api/projects/{slug}/secrets", api_token=api_token)
    assert "secrets" in secrets_list, "Secrets list failed"

    secret_delete = request_json(base_url, "DELETE", f"/api/projects/{slug}/secrets/openai/api_key", api_token=api_token)
    assert secret_delete.get("ok") is True, "Secret deletion failed"
    print("           ✓ Secrets CRUD complete")

    # Exports list
    print("[contract-smoke] Exports list")
    exports = request_json(base_url, "GET", f"/api/projects/{slug}/exports", api_token=api_token)
    assert "project_exports" in exports or "exports" in exports, "Exports response should include exports list"
    print("           ✓ Exports retrieved")

    print()
    print(f"[contract-smoke] ✓ All tests passed! project={slug}")
    print(f"[contract-smoke] Created resources: template={template_id}, session={session_id}, instruction={instr_id}")
    print()
    print("[contract-smoke] NOTE: To enable auth for local development, set:")
    print("                     KROMA_API_AUTH_DEV_BYPASS=true")


if __name__ == "__main__":
    main()
