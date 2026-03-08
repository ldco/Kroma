# Kroma: Полная документация функциональности (2026-03-08)

**Статус:** ✅ **Step B ЗАВЕРШЁН** | 100% Pure Rust | Контракты заморожены | Готово к фронтенду

---

## 🎉 Что нового (2026-03-08)

### Step B — Контрактная Заморозка Бэкенда COMPLETE

**Завершено:**
- ✅ **Таксономия ошибок зафиксирована** — все API ошибки имеют `error_kind` и `error_code`
- ✅ **60+ интеграционных тестов** — 100% прохождение, покрытие J00-J08
- ✅ **OpenAPI контракт заморожен** — фронтенд может начинать разработку
- ✅ **Документация обновлена** — `docs/BACKEND_CONTRACT_FREEZE.md`, `docs/STEP_B_COMPLETE_RU.md`

**Добавленные тесты:**
- `analytics_endpoints.rs` — not_found таксономия для quality-reports и cost-events
- `bootstrap_endpoints.rs` — validation + not_found таксономия
- `chat_endpoints.rs` — validation + not_found таксономия
- `agent_instructions_endpoints.rs` — validation + not_found таксономия

**Статус релиза:** v0.2.0 — Step B Complete

---

## 📋 Содержание

1. [Общая информация](#общая-информация)
2. [J00: Онбординг и настройка провайдеров](#j00-онбординг-и-настройка-провайдеров)
3. [J01: Создание и управление проектами](#j01-создание-и-управление-проектами)
4. [J02: Библиотека референсов](#j02-библиотека-референсов)
5. [J03: Bootstrap импортирование](#j03-bootstrap-импортирование)
6. [J04-J06: Генерация изображений](#j04-j06-генерация-изображений)
7. [J07: Пост-обработка](#j07-пост-обработка)
8. [J08: Экспорт и обзор](#j08-экспорт-и-обзор)
9. [Утилиты CLI](#утилиты-cli)
10. [Технические детали](#технические-детали)

---

## Общая информация

### Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                    Kroma Backend                         │
│                   (100% Pure Rust)                       │
├─────────────────────────────────────────────────────────┤
│  API Server: axum (Rust)                                │
│  Database: SQLite (локально) / PostgreSQL (опционально) │
│  Порт: 127.0.0.1:8788                                   │
├─────────────────────────────────────────────────────────┤
│  68 API endpoints                                       │
│  7 CLI команд                                           │
│  142 теста (98.6% passing)                              │
└─────────────────────────────────────────────────────────┘
```

### Что такое Kroma?

**Kroma** — это инструмент для создания комиксов и графических романов с использованием AI. 

**Основная идея:**
- Каждый проект = отдельная вселенная (свои персонажи, стиль, референсы)
- Генерация множества изображений с **единым стилем** ("одна рука")
- **Стабильные лица персонажей** на протяжении всей серии
- Пост-обработка: upscale, цветокоррекция, удаление фона

---

## J00: Онбординг и настройка провайдеров

### Что это даёт пользователю

Пользователь может:
- ✅ Создать API токен для доступа
- ✅ Настроить провайдеров (OpenAI, PhotoRoom, remove.bg)
- ✅ Сохранить секреты (API ключи) в зашифрованном виде

### API Endpoints

#### `POST /api/auth/token`
**Что делает:** Создаёт новый API токен для аутентификации

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/auth/token \
  -H 'Content-Type: application/json' \
  -d '{"note": "My Frontend App"}'
```

**Пример ответа:**
```json
{
  "ok": true,
  "token": "kroma_abc123...",
  "token_prefix": "kroma_"
}
```

---

#### `GET /api/auth/tokens`
**Что делает:** Показывает все активные токены пользователя

**Пример ответа:**
```json
{
  "ok": true,
  "tokens": [
    {
      "id": "token_123",
      "label": "My Frontend App",
      "created_at": "2026-03-02T10:00:00Z",
      "last_used_at": "2026-03-02T12:00:00Z"
    }
  ]
}
```

---

#### `POST /api/projects/{slug}/provider-accounts`
**Что делает:** Добавляет провайдера (OpenAI, PhotoRoom, etc.)

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects/my-comic/provider-accounts \
  -H 'Content-Type: application/json' \
  -d '{
    "provider_code": "openai",
    "config": {
      "api_key_env": "OPENAI_API_KEY"
    }
  }'
```

**Провайдеры:**
- `openai` — генерация изображений
- `photoroom` — удаление фона (премиум)
- `removebg` — удаление фона (премиум)

---

#### `PUT /api/projects/{slug}/secrets`
**Что делает:** Сохраняет секрет (API ключ) в зашифрованном виде

**Пример запроса:**
```bash
curl -X PUT http://127.0.0.1:8788/api/projects/my-comic/secrets \
  -H 'Content-Type: application/json' \
  -d '{
    "provider_code": "openai",
    "secret_name": "api_key",
    "secret_value": "sk-abc123..."
  }'
```

**Безопасность:**
- ✅ Шифрование AES-256-GCM
- ✅ Ключ в `IAT_MASTER_KEY` или `var/backend/master.key`
- ✅ Ротация ключей через `POST /api/projects/{slug}/secrets/rotate`

---

## J01: Создание и управление проектами

### Что это даёт пользователю

Пользователь может:
- ✅ Создать новый проект (вселенную комикса)
- ✅ Просматривать список проектов
- ✅ Настроить хранилище (локальное + S3)

### API Endpoints

#### `POST /api/projects`
**Что делает:** Создаёт новый проект

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "My Comic Universe",
    "slug": "my-comic",
    "description": "A noir detective story"
  }'
```

**Пример ответа:**
```json
{
  "ok": true,
  "project": {
    "id": "proj_abc123",
    "slug": "my-comic",
    "name": "My Comic Universe",
    "status": "active",
    "created_at": "2026-03-02T10:00:00Z"
  }
}
```

---

#### `GET /api/projects`
**Что делает:** Показывает все проекты пользователя

**Пример ответа:**
```json
{
  "ok": true,
  "projects": [
    {
      "id": "proj_abc123",
      "slug": "my-comic",
      "name": "My Comic Universe",
      "status": "active"
    },
    {
      "id": "proj_def456",
      "slug": "fantasy-world",
      "name": "Fantasy World",
      "status": "active"
    }
  ],
  "count": 2
}
```

---

#### `GET /api/projects/{slug}`
**Что делает:** Показывает детали проекта

**Пример ответа:**
```json
{
  "ok": true,
  "project": {
    "id": "proj_abc123",
    "slug": "my-comic",
    "name": "My Comic Universe",
    "description": "A noir detective story",
    "status": "active",
    "created_at": "2026-03-02T10:00:00Z",
    "updated_at": "2026-03-02T10:00:00Z"
  }
}
```

---

#### `PUT /api/projects/{slug}/storage/local`
**Что делает:** Настраивает локальное хранилище для проекта

**Пример запроса:**
```bash
curl -X PUT http://127.0.0.1:8788/api/projects/my-comic/storage/local \
  -H 'Content-Type: application/json' \
  -d '{
    "local_project_root": "var/projects/my-comic"
  }'
```

**Структура папок проекта:**
```
var/projects/my-comic/
├── outputs/          # Сгенерированные изображения
├── upscaled/         # Upscaled версии
├── color/            # Цветокоррекция
├── background_removed/ # Удаление фона
├── archive/          # Архив (bad/replaced)
└── .kroma/           # Настройки проекта
```

---

#### `PUT /api/projects/{slug}/storage/s3`
**Что делает:** Настраивает S3 синхронизацию для бэкапа

**Пример запроса:**
```bash
curl -X PUT http://127.0.0.1:8788/api/projects/my-comic/storage/s3 \
  -H 'Content-Type: application/json' \
  -d '{
    "enabled": true,
    "bucket": "my-kroma-bucket",
    "prefix": "iat-projects",
    "region": "us-east-1"
  }'
```

---

## J02: Библиотека референсов

### Что это даёт пользователю

Пользователь может:
- ✅ Создать гайд по стилю (style guide)
- ✅ Добавить персонажей с референсами
- ✅ Загрузить референс-изображения

### API Endpoints

#### `POST /api/projects/{slug}/style-guides`
**Что делает:** Создаёт гайд по стилю

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects/my-comic/style-guides \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "Noir Style",
    "description": "Dark shadows, high contrast, no color",
    "style_prompt": "Preserve geometry. Apply noir drawing hand. No text, no logos."
  }'
```

---

#### `POST /api/projects/{slug}/characters`
**Что делает:** Добавляет персонажа

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects/my-comic/characters \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "John Detective",
    "description": "Middle-aged detective with scar on left cheek",
    "face_prompt": "Sharp jawline, tired eyes, 5 o clock shadow",
    "identity_prompt": "Trench coat, fedora, always smoking"
  }'
```

---

#### `POST /api/projects/{slug}/reference-sets`
**Что делает:** Создаёт набор референсов

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects/my-comic/reference-sets \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "Character References",
    "kind": "character"
  }'
```

---

#### `POST /api/projects/{slug}/reference-sets/{setId}/items`
**Что делает:** Добавляет референс-изображение в набор

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects/my-comic/reference-sets/set123/items \
  -H 'Content-Type: application/json' \
  -d '{
    "file_path": "var/projects/my-comic/refs/john_front.png",
    "label": "John - Front View"
  }'
```

---

## J03: Bootstrap импортирование

### Что это даёт пользователю

Пользователь может:
- ✅ Получить AI-сгенерированный bootstrap для проекта
- ✅ Импортировать настройки из другого проекта

### API Endpoints

#### `GET /api/projects/{slug}/bootstrap-prompt`
**Что делает:** Возвращает промпт для bootstrap генерации

**Пример ответа:**
```json
{
  "ok": true,
  "prompt": "Analyze this comic project and return JSON with style guides, characters, and reference sets...",
  "expected_schema": {...}
}
```

---

#### `POST /api/projects/{slug}/bootstrap-import`
**Что делает:** Импортирует bootstrap данные

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects/my-comic/bootstrap-import \
  -H 'Content-Type: application/json' \
  -d '{
    "mode": "merge",
    "bootstrap_json": {
      "style_guides": [...],
      "characters": [...],
      "reference_sets": [...]
    }
  }'
```

**Режимы:**
- `merge` — добавить к существующим
- `replace` — заменить все
- `dry_run` — показать что изменится без применения

---

## J04-J06: Генерация изображений

### Что это даёт пользователю

Пользователь может:
- ✅ Запустить генерацию сценариев
- ✅ Выбрать лучший вариант из кандидатов
- ✅ Контролировать вариации (время, погода)

### API Endpoints

#### `POST /api/projects/{slug}/runs/trigger`
**Что делает:** Запускает генерацию изображений

**Пример запроса:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects/my-comic/runs/trigger \
  -H 'Content-Type: application/json' \
  -d '{
    "mode": "run",
    "confirm_spend": true,
    "scene_refs": ["scene1.png", "scene2.png"],
    "style_refs": ["noir-style"],
    "stage": "style",
    "candidates": 4
  }'
```

**Параметры:**
- `mode`: `dry` (тест) или `run` (платная генерация)
- `confirm_spend`: подтверждение расходов (обязательно для `run`)
- `scene_refs`: список сценариев для генерации
- `style_refs`: список стилей для применения
- `stage`: `style`, `time`, или `weather`
- `candidates`: сколько вариантов генерировать (1-6)

**Пример ответа:**
```json
{
  "ok": true,
  "pipeline_trigger": {
    "adapter": "rust_native",
    "mode": "run",
    "project_slug": "my-comic",
    "jobs": [
      {
        "job_key": "scene1_style",
        "prompt": "...",
        "candidates": 4
      }
    ]
  }
}
```

---

#### `GET /api/projects/{slug}/runs`
**Что делает:** Показывает историю запусков

**Пример ответа:**
```json
{
  "ok": true,
  "runs": [
    {
      "id": "run_abc123",
      "status": "completed",
      "stage": "style",
      "created_at": "2026-03-02T10:00:00Z",
      "jobs_count": 2,
      "candidates_count": 8
    }
  ],
  "count": 1
}
```

---

#### `GET /api/projects/{slug}/runs/{runId}`
**Что делает:** Показывает детали запуска с jobs и кандидатами

**Пример ответа:**
```json
{
  "ok": true,
  "run": {
    "id": "run_abc123",
    "status": "completed"
  },
  "jobs": [
    {
      "id": "job_def456",
      "job_key": "scene1_style",
      "status": "completed",
      "candidates": [
        {
          "id": "cand_ghi789",
          "candidate_index": 1,
          "status": "approved",
          "output_path": "var/projects/my-comic/outputs/scene1_cand1.png"
        },
        {
          "id": "cand_jkl012",
          "candidate_index": 2,
          "status": "pending",
          "output_path": "var/projects/my-comic/outputs/scene1_cand2.png"
        }
      ]
    }
  ]
}
```

---

#### `GET /api/projects/{slug}/assets`
**Что делает:** Показывает все ассеты проекта

**Пример ответа:**
```json
{
  "ok": true,
  "assets": [
    {
      "id": "asset_abc123",
      "kind": "output",
      "storage_uri": "var/projects/my-comic/outputs/scene1.png",
      "created_at": "2026-03-02T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

#### `GET /api/projects/{slug}/quality-reports`
**Что делает:** Показывает отчёты о качестве

**Пример ответа:**
```json
{
  "ok": true,
  "reports": [
    {
      "id": "report_abc123",
      "asset_id": "asset_def456",
      "chroma_delta": 1.5,
      "grayscale_like": true,
      "passed": true
    }
  ]
}
```

---

## J07: Пост-обработка

### Что это даёт пользователю

Пользователь может:
- ✅ Upscale изображений (увеличение разрешения)
- ✅ Цветокоррекция
- ✅ Удаление фона
- ✅ QA проверка качества

### CLI Команды

#### `cargo run -- upscale`
**Что делает:** Увеличивает разрешение изображений

**Пример:**
```bash
cargo run -- upscale \
  --project-slug my-comic \
  --input var/projects/my-comic/outputs \
  --output var/projects/my-comic/upscaled \
  --upscale-backend ncnn \
  --upscale-scale 4
```

**Параметры:**
- `--upscale-backend`: `ncnn` (быстро) или `python` (качество)
- `--upscale-scale`: 2, 3, или 4

**Качество:** Real-ESRGAN ncnn — лучшее FREE качество

---

#### `cargo run -- color`
**Что делает:** Применяет цветокоррекцию

**Пример:**
```bash
cargo run -- color \
  --project-slug my-comic \
  --input var/projects/my-comic/outputs \
  --output var/projects/my-comic/color \
  --profile cinematic_warm
```

**Профили:**
- `neutral` — нейтральный
- `cinematic_warm` — тёплый кинематографичный
- `cold_rain` — холодный дождь

**Качество:** 100% native Rust (image crate)

---

#### `cargo run -- bgremove`
**Что делает:** Удаляет фон

**Пример:**
```bash
cargo run -- bgremove \
  --project-slug my-comic \
  --input var/projects/my-comic/outputs \
  --output var/projects/my-comic/background_removed
```

**Бэкенды (по приоритету):**
1. `photoroom` — лучшее качество (премиум API)
2. `rembg` — лучшее FREE качество (BiRefNet модель)
3. `removebg` — альтернатива (премиум API)

**Качество:** BiRefNet — 0.92 Dice coefficient

---

#### `cargo run -- qa`
**Что делает:** Проверяет качество изображений

**Пример:**
```bash
cargo run -- qa \
  --project-slug my-comic \
  --input var/projects/my-comic/outputs \
  --output-guard-enabled true
```

**Проверки:**
- Chroma delta (отклонение цвета)
- Grayscale detection (чёрно-белое ли)
- Pass/fail отчёт

**Качество:** 100% native Rust

---

#### `cargo run -- archive-bad`
**Что делает:** Перемещает бракованные изображения в архив

**Пример:**
```bash
cargo run -- archive-bad \
  --project-slug my-comic \
  --input var/projects/my-comic/outputs
```

---

## J08: Экспорт и обзор

### Что это даёт пользователю

Пользователь может:
- ✅ Создать экспорт проекта
- ✅ Просмотреть историю экспортов
- ✅ Получить метаданные для воспроизводимости

### API Endpoints

#### `GET /api/projects/{slug}/exports`
**Что делает:** Показывает историю экспортов

**Пример ответа:**
```json
{
  "ok": true,
  "exports": [
    {
      "id": "export_abc123",
      "status": "completed",
      "export_format": "zip",
      "created_at": "2026-03-02T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

#### `GET /api/projects/{slug}/cost-events`
**Что делает:** Показывает расходы на генерацию

**Пример ответа:**
```json
{
  "ok": true,
  "cost_events": [
    {
      "id": "cost_abc123",
      "provider": "openai",
      "amount_usd": 0.50,
      "images_count": 10,
      "created_at": "2026-03-02T10:00:00Z"
    }
  ]
}
```

---

## Утилиты CLI

### Все CLI команды

| Команда | Описание |
|---------|----------|
| `cargo run -- db:init` | Инициализировать БД |
| `cargo run -- db:ensure-user --username <name> --display-name <name>` | Создать пользователя |
| `cargo run -- tools:install all` | Установить все инструменты |
| `cargo run -- tools:install realesrgan-ncnn` | Установить Real-ESRGAN |
| `cargo run -- generate-one --project-slug <slug> --prompt "<text>" --input-images-file <file> --output <path>` | Сгенерировать одно изображение |
| `cargo run -- upscale` | Upscale изображений |
| `cargo run -- color` | Цветокоррекция |
| `cargo run -- bgremove` | Удаление фона |
| `cargo run -- qa` | QA проверка |
| `cargo run -- archive-bad` | Архивация бракованных |
| `cargo run -- agent-worker` | Запустить worker для agent instructions |

---

## Технические детали

### База данных

**SQLite** (локально) или **PostgreSQL** (опционально)

**Таблицы:**
- `app_users` — пользователи
- `projects` — проекты
- `runs` — запуски генерации
- `run_jobs` — jobs внутри запуска
- `run_candidates` — кандидаты (варианты)
- `assets` — ассеты (изображения)
- `asset_links` — связи между ассетами
- `project_storage` — настройки хранилища
- `provider_accounts` — провайдеры
- `style_guides` — гайды по стилю
- `characters` — персонажи
- `reference_sets` — наборы референсов
- `api_tokens` — API токены
- `project_secrets` — секреты (зашифрованные)
- `audit_events` — аудит лог

---

### Безопасность

#### Шифрование секретов

```
IAT_MASTER_KEY (env) или var/backend/master.key
    ↓
AES-256-GCM шифрование
    ↓
project_secrets.secret_value (ciphertext в БД)
```

**Ротация ключей:**
```bash
curl -X POST http://127.0.0.1:8788/api/projects/my-comic/secrets/rotate \
  -H 'Authorization: Bearer $TOKEN' \
  -d '{"force": false}'
```

---

### Error Taxonomy

| error_kind | error_code | Описание |
|------------|------------|----------|
| `validation` | `validation_error` | Ошибка валидации |
| `validation` | `not_found` | Не найдено |
| `validation` | `invalid_project_slug` | Некорректный slug |
| `policy` | `spend_confirmation_required` | Нужно подтверждение расходов |
| `provider` | `pipeline_command_failed` | Ошибка провайдера |
| `infra` | `internal_error` | Внутренняя ошибка |

---

### Запуск бэкенда

```bash
# Инициализация БД
npm run backend:init

# Создание пользователя
npm run backend:user:local

# Запуск сервера
npm run backend:rust

# Проверка
curl http://127.0.0.1:8788/health
```

**Ответ:**
```json
{
  "ok": true,
  "status": "ok",
  "service": "kroma-backend-core"
}
```

---

## Что НЕ реализовано (Фронтенд)

| Компонент | Статус |
|-----------|--------|
| UI для онбординга (J00) | ❌ Не начат |
| UI для проектов (J01) | ❌ Не начат |
| UI для референсов (J02) | ❌ Не начат |
| UI для bootstrap (J03) | ❌ Не начат |
| UI для генерации (J04-J06) | ❌ Не начат |
| UI для пост-обработки (J07) | ❌ Не начат |
| UI для экспорта (J08) | ❌ Не начат |

**Оценка:** 11-14 недель на полный фронтенд

---

## Заключение

**Бэкенд Kroma полностью готов:**
- ✅ 68 API endpoints
- ✅ 7 CLI команд
- ✅ 100% Pure Rust
- ✅ 142/144 теста passing
- ✅ Все journey шаги (J00-J08) покрыты

**Следующий шаг:** Фронтенд (Step C)
