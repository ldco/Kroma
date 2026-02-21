#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:8787}"
PROJECT_SLUG="${2:-contract_demo}"

echo "[contract-smoke] health"
curl -fsS "${BASE_URL}/health" >/tmp/iat_contract_health.json

echo "[contract-smoke] upsert project"
curl -fsS -X POST "${BASE_URL}/api/projects" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"Contract Demo\",\"slug\":\"${PROJECT_SLUG}\"}" >/tmp/iat_contract_project.json

echo "[contract-smoke] create session"
SESSION_ID="$(curl -fsS -X POST "${BASE_URL}/api/projects/${PROJECT_SLUG}/chat/sessions" \
  -H "Content-Type: application/json" \
  -d '{"title":"Contract session"}' | python3 -c 'import sys,json;print(json.load(sys.stdin)["session"]["id"])')"

echo "[contract-smoke] add message"
curl -fsS -X POST "${BASE_URL}/api/projects/${PROJECT_SLUG}/chat/sessions/${SESSION_ID}/messages" \
  -H "Content-Type: application/json" \
  -d '{"role":"user","content_text":"Create instruction"}' >/tmp/iat_contract_msg.json

echo "[contract-smoke] create instruction"
INSTR_ID="$(curl -fsS -X POST "${BASE_URL}/api/projects/${PROJECT_SLUG}/agent/instructions" \
  -H "Content-Type: application/json" \
  -d '{"instruction_type":"pipeline_run","dispatch_to_agent":false,"payload_json":{"stage":"style","candidates":2}}' | python3 -c 'import sys,json;print(json.load(sys.stdin)["instruction"]["id"])')"

echo "[contract-smoke] events"
curl -fsS "${BASE_URL}/api/projects/${PROJECT_SLUG}/agent/instructions/${INSTR_ID}/events" >/tmp/iat_contract_events.json

echo "[contract-smoke] voice stt + tts"
STT_REQ_ID="$(curl -fsS -X POST "${BASE_URL}/api/projects/${PROJECT_SLUG}/voice/stt" \
  -H "Content-Type: application/json" \
  -d "{\"session_id\":\"${SESSION_ID}\",\"provider_code\":\"mock_stt\",\"transcript_text\":\"hello\"}" | python3 -c 'import sys,json;print(json.load(sys.stdin)["request"]["id"])')"
curl -fsS "${BASE_URL}/api/projects/${PROJECT_SLUG}/voice/requests/${STT_REQ_ID}" >/tmp/iat_contract_voice_stt.json

TTS_REQ_ID="$(curl -fsS -X POST "${BASE_URL}/api/projects/${PROJECT_SLUG}/voice/tts" \
  -H "Content-Type: application/json" \
  -d "{\"session_id\":\"${SESSION_ID}\",\"provider_code\":\"mock_tts\",\"text\":\"ok\"}" | python3 -c 'import sys,json;print(json.load(sys.stdin)["request"]["id"])')"
curl -fsS "${BASE_URL}/api/projects/${PROJECT_SLUG}/voice/requests/${TTS_REQ_ID}" >/tmp/iat_contract_voice_tts.json

echo "[contract-smoke] secrets"
curl -fsS -X POST "${BASE_URL}/api/projects/${PROJECT_SLUG}/secrets" \
  -H "Content-Type: application/json" \
  -d '{"provider_code":"openai","secret_name":"api_key","secret_value":"sk-test-contract-123456"}' >/tmp/iat_contract_secret_set.json
curl -fsS "${BASE_URL}/api/projects/${PROJECT_SLUG}/secrets" >/tmp/iat_contract_secret_list.json
curl -fsS -X DELETE "${BASE_URL}/api/projects/${PROJECT_SLUG}/secrets/openai/api_key" >/tmp/iat_contract_secret_delete.json

echo "[contract-smoke] ok project=${PROJECT_SLUG} session=${SESSION_ID} instruction=${INSTR_ID}"
