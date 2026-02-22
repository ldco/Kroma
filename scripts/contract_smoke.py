#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import urllib.error
import urllib.request


def request_json(base_url: str, method: str, path: str, payload: dict | None = None) -> dict:
    data = None
    headers = {"Content-Type": "application/json"}
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
        raise SystemExit(f"{method} {path} failed: {exc}") from exc
    if not raw:
        return {}
    return json.loads(raw)


def main():
    parser = argparse.ArgumentParser(description="Backend API contract smoke test")
    parser.add_argument("--base-url", default="http://127.0.0.1:8787", help="Backend API base URL")
    parser.add_argument("--project-slug", default="contract_demo", help="Project slug to use")
    args = parser.parse_args()

    base_url = args.base_url.rstrip("/")
    slug = args.project_slug.strip() or "contract_demo"

    print("[contract-smoke] health")
    request_json(base_url, "GET", "/health")

    print("[contract-smoke] upsert project")
    request_json(base_url, "POST", "/api/projects", {"name": "Contract Demo", "slug": slug})

    print("[contract-smoke] prompt templates CRUD")
    created_template = request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/prompt-templates",
        {"name": "default-shot", "template_text": "A cinematic shot of {subject}"},
    )
    template_id = created_template["prompt_template"]["id"]
    request_json(base_url, "GET", f"/api/projects/{slug}/prompt-templates")
    request_json(base_url, "GET", f"/api/projects/{slug}/prompt-templates/{template_id}")
    request_json(
        base_url,
        "PUT",
        f"/api/projects/{slug}/prompt-templates/{template_id}",
        {"template_text": "A cinematic close-up of {subject}"},
    )
    request_json(base_url, "DELETE", f"/api/projects/{slug}/prompt-templates/{template_id}")

    print("[contract-smoke] create session")
    sess = request_json(base_url, "POST", f"/api/projects/{slug}/chat/sessions", {"title": "Contract session"})
    session_id = sess["session"]["id"]

    print("[contract-smoke] add message")
    request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/chat/sessions/{session_id}/messages",
        {"role": "user", "content_text": "Create instruction"},
    )

    print("[contract-smoke] create instruction")
    ins = request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/agent/instructions",
        {"instruction_type": "pipeline_run", "dispatch_to_agent": False, "payload_json": {"stage": "style", "candidates": 2}},
    )
    instr_id = ins["instruction"]["id"]

    print("[contract-smoke] events")
    request_json(base_url, "GET", f"/api/projects/{slug}/agent/instructions/{instr_id}/events")

    print("[contract-smoke] secrets")
    request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/secrets",
        {"provider_code": "openai", "secret_name": "api_key", "secret_value": "sk-test-contract-123456"},
    )
    request_json(base_url, "GET", f"/api/projects/{slug}/secrets")
    request_json(base_url, "DELETE", f"/api/projects/{slug}/secrets/openai/api_key")

    print("[contract-smoke] export + exports read")
    export_result = request_json(
        base_url,
        "POST",
        f"/api/projects/{slug}/export",
        {"include_files": False, "output": f"var/exports/{slug}_contract_export.tar.gz"},
    )
    exports = request_json(base_url, "GET", f"/api/projects/{slug}/exports")
    export_asset_id = export_result.get("export_asset_id")
    export_id = None
    for item in exports.get("project_exports", []):
        if item.get("export_asset_id") == export_asset_id:
            export_id = item.get("id")
            break
    if not export_id:
        raise SystemExit("Failed to resolve export id from exports list")
    request_json(base_url, "GET", f"/api/projects/{slug}/exports/{export_id}")

    print(
        f"[contract-smoke] ok project={slug} template={template_id} export={export_id} session={session_id} instruction={instr_id}"
    )


if __name__ == "__main__":
    main()
