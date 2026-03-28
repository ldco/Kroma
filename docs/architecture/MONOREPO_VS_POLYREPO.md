# Monorepo vs Polyrepo: Kroma Architecture

**Вопрос:** Нужно ли два репозитория (backend + frontend) или один (monorepo)?

**Ответ:** **Один репозиторий (Monorepo) ✅** — лучше для вашего случая.

---

## 📊 Сравнение Подходов

### Option 1: Monorepo (Рекомендуется ✅)

```
Kroma/
├── app/                    # Весь код в одном месте
│   ├── src-tauri/          # Rust backend
│   ├── frontend/           # Nuxt frontend
│   ├── docs/               # Документация
│   └── package.json        # Общие scripts
│
├── .github/                # CI/CD workflows
├── README.md
└── ...
```

**Преимущества:**
- ✅ Один `git clone` для всего проекта
- ✅ Atomic commits (backend + frontend вместе)
- ✅ Легко делать full-stack изменения
- ✅ Одна версия для всего
- ✅ Проще CI/CD (один pipeline)
- ✅ Общая документация
- ✅ Нет проблем с версиями API

**Недостатки:**
- ⚠️ Больше размер репозитория
- ⚠️ Нужно управлять build order
- ⚠️ CI может быть медленнее

---

### Option 2: Polyrepo (Не рекомендуется ❌)

```
Kroma-Backend/              Kroma-Frontend/
├── src-tauri/              ├── frontend/
├── docs/                   ├── docs/
└── README.md               └── README.md
```

**Преимущества:**
- ✅ Меньшие репозитории
- ✅ Раздельный CI/CD
- ✅ Разный access control

**Недостатки:**
- ❌ Сложно координировать изменения
- ❌ Нужно управлять версиями API
- ❌ Atomic commits невозможны
- ❌ Две документации
- ❌ Проблемы совместимости
- ❌ Сложнее для full-stack разработки

---

## 🎯 Почему Monorepo Лучше для Kroma

### 1. Journey-Driven Разработка

Вы разрабатываете по journey steps (J00-J08):

```
J01: Проекты
├── Backend: POST /api/projects ✅
├── Backend: GET /api/projects ✅
├── Frontend: ProjectsPage.vue ✅
└── Frontend: ProjectCard.vue ✅
```

**В monorepo:**
```bash
git commit -m "feat(J01): add projects CRUD"
# Включает backend + frontend изменения вместе
```

**В polyrepo:**
```bash
# Repo 1
cd Kroma-Backend
git commit -m "feat: add projects API"
git push

# Repo 2
cd Kroma-Frontend
git commit -m "feat: add projects UI"
git push
```

**Проблема:** Если что-то сломалось, нужно откатывать два репозитория!

---

### 2. API Contract Evolution

**Сценарий:** Нужно изменить API endpoint

**Monorepo:**
```bash
# Один PR включает всё:
1. Изменить backend API
2. Изменить frontend API client
3. Обновить тесты
4. Задеплоить вместе

git commit -m "refactor: change projects API shape"
```

**Polyrepo:**
```bash
# Шаг 1: Backend PR
cd Kroma-Backend
# Изменить API
git commit -m "breaking: change projects API"
git push

# Шаг 2: Frontend PR (должен ждать backend deploy)
cd Kroma-Frontend
# Обновить API client
git commit -m "fix: update projects API client"
git push

# Проблема: Окно несовместимости!
```

---

### 3. Version Management

**Monorepo:**
```json
{
  "name": "kroma",
  "version": "0.1.0"  // Одна версия для всего
}
```

**Polyrepo:**
```json
// Kroma-Backend/package.json
{
  "name": "kroma-backend",
  "version": "0.1.0"
}

// Kroma-Frontend/package.json
{
  "name": "kroma-frontend",
  "version": "0.1.0"
}
```

**Проблема:** Как узнать какая версия frontend совместима с какой backend?

---

## 📁 Recommended Monorepo Structure

```
app/
├── src-tauri/                    # Rust Backend
│   ├── src/
│   │   ├── api/                  # API routes
│   │   ├── db/                   # Database layer
│   │   ├── pipeline/             # Pipeline logic
│   │   └── main.rs               # Entry point
│   ├── tests/                    # Backend tests
│   ├── Cargo.toml
│   └── Cargo.lock
│
├── frontend/                     # Nuxt Frontend
│   ├── src-tauri/                # Tauri desktop wrapper
│   │   ├── src/
│   │   │   ├── main.rs           # Tauri + embedded backend
│   │   │   └── commands.rs       # Tauri commands
│   │   ├── tauri.conf.json       # Tauri config
│   │   └── Cargo.toml
│   │
│   ├── composables/              # Vue composables
│   ├── components/               # UI components
│   ├── pages/                    # Nuxt pages (J00-J08)
│   ├── nuxt.config.ts
│   ├── package.json
│   └── ...
│
├── docs/                         # Общая документация
│   ├── ROADMAP.md
│   ├── FUNCTIONALITY_COMPLETE_RU.md
│   ├── HYBRID_ARCHITECTURE_PLAN.md
│   ├── PARTIAL_TAURI_NUXT.md
│   └── RUST_BACKEND_DESKTOP_SERVER.md
│
├── .github/                      # CI/CD
│   └── workflows/
│       ├── backend-test.yml      # Backend tests
│       ├── frontend-test.yml     # Frontend tests
│       ├── desktop-build.yml     # Tauri build
│       └── web-deploy.yml        # Web deploy
│
├── package.json                  # Общие npm scripts
├── README.md
└── .gitignore
```

---

## 🔄 Workflow: Full-Stack Feature

### Сценарий: Добавить J02 (References)

**Monorepo Workflow:**

```bash
# 1. Создать feature branch
git checkout -b feat/j02-references

# 2. Backend изменения
cd src-tauri
# Добавить API endpoints в src/api/reference_sets.rs
# Добавить тесты в tests/reference_sets_endpoints.rs
cargo test

# 3. Frontend изменения
cd ../frontend
# Добавить pages/references.vue
# Добавить components/ReferenceCard.vue
npm run build

# 4. Закоммитить всё вместе
cd ..
git add src-tauri/src/api/reference_sets.rs
git add src-tauri/tests/reference_sets_endpoints.rs
git add frontend/pages/references.vue
git add frontend/components/ReferenceCard.vue

git commit -m "feat(J02): add reference sets CRUD

Backend:
- POST /api/projects/{slug}/reference-sets
- GET /api/projects/{slug}/reference-sets
- PUT/DELETE /api/projects/{slug}/reference-sets/{id}

Frontend:
- References page with list/create/edit
- ReferenceCard component
- File upload integration

Tests:
- Backend: reference_sets_endpoints.rs (8 tests)
- Frontend: references.spec.ts (e2e)"

git push origin feat/j02-references

# 5. Pull Request (один PR для всего)
# Reviewer видит все изменения вместе
```

**Преимущества:**
- ✅ Один PR для review
- ✅ Тесты backend + frontend вместе
- ✅ Atomic commit (всё или ничего)
- ✅ Легко откатить если что-то сломалось

---

## 🚀 CI/CD для Monorepo

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]

jobs:
  # Backend tests
  backend-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cd src-tauri && cargo test

  # Frontend tests
  frontend-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: cd frontend && npm ci
      - run: cd frontend && npm run test

  # Desktop build (только для master)
  desktop-build:
    if: github.ref == 'refs/heads/master'
    needs: [backend-test, frontend-test]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: actions-rs/toolchain@v1
      - run: cd frontend && npm ci
      - run: cd frontend && npm run tauri build

  # Web deploy (только для master)
  web-deploy:
    if: github.ref == 'refs/heads/master'
    needs: [backend-test, frontend-test]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: cd frontend && npm ci
      - run: cd frontend && npm run build
      # Deploy to Vercel/Netlify/VPS
```

**Преимущества:**
- ✅ Все тесты запускаются вместе
- ✅ Desktop + web build координированы
- ✅ Один статус CI для всего PR

---

## 📦 Package Management

### Nuxt Frontend

```json
// frontend/package.json
{
  "name": "@kroma/frontend",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "nuxt dev",
    "build": "nuxt build",
    "tauri": "tauri",
    "tauri:build": "tauri build"
  }
}
```

### Rust Backend

```toml
# src-tauri/Cargo.toml
[package]
name = "kroma-backend-core"
version = "0.1.0"
edition = "2021"

[lib]
name = "kroma_backend_core"
path = "src/lib.rs"
```

### Корневой package.json

```json
// package.json (в корне app/)
{
  "name": "kroma",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "backend:test": "cd src-tauri && cargo test",
    "backend:build": "cd src-tauri && cargo build",
    "backend:run": "cd src-tauri && cargo run",
    "frontend:dev": "cd frontend && npm run dev",
    "frontend:build": "cd frontend && npm run build",
    "desktop:build": "cd frontend && npm run tauri build",
    "test": "npm run backend:test && npm run frontend:test",
    "build": "npm run backend:build && npm run frontend:build"
  }
}
```

**Использование:**
```bash
# Запустить всё
npm run test

# Запустить backend
npm run backend:run

# Запустить frontend dev
npm run frontend:dev

# Desktop build
npm run desktop:build
```

---

## 🔐 Access Control

**Monorepo:** Все имеют доступ ко всему

**Polyrepo:** Можно ограничить доступ

**Для Kroma:** Monorepo лучше, потому что:
- Маленькая команда (1-2 человека)
- Full-stack разработка
- Нет необходимости в разном access control

---

## 📊 Size Comparison

**Monorepo:**
```
$ du -sh .
2.1 GB  # Включая node_modules, target, .git
```

**Polyrepo:**
```
Kroma-Backend:  800 MB
Kroma-Frontend: 1.5 GB
Total:          2.3 GB  # Больше из-за дублирования .git
```

**Вывод:** Monorepo даже меньше!

---

## ⚠️ Gotchas и Решения

### 1. Build Order

**Проблема:** Frontend зависит от backend API types

**Решение:** Генерировать types из OpenAPI spec

```bash
# scripts/generate-types.sh
cd src-tauri
# Экспорт OpenAPI spec
cargo run -- export-openapi > ../frontend/types/api.json

cd ../frontend
# Генерировать TypeScript types
npx openapi-typescript types/api.json -o types/api.ts
```

---

### 2. CI Время

**Проблема:** CI запускает всё для каждого PR

**Решение:** Path-based triggers

```yaml
# .github/workflows/ci.yml

jobs:
  backend-test:
    # Запускать только если backend изменился
    if: |
      github.event_name == 'push' ||
      contains(github.event.pull_request.changed_files, 'src-tauri/')
    
  frontend-test:
    # Запускать только если frontend изменился
    if: |
      github.event_name == 'push' ||
      contains(github.event.pull_request.changed_files, 'frontend/')
```

---

### 3. Git History

**Проблема:** Большая история = медленный git

**Решение:** Shallow clone в CI

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 1  # Только последний commit
```

---

## 🎯 Рекомендация

**Используйте Monorepo структуру:**

```
app/
├── src-tauri/          # Rust backend
├── frontend/           # Nuxt frontend
├── docs/               # Документация
├── .github/            # CI/CD
└── package.json        # Общие scripts
```

**Преимущества для Kroma:**
- ✅ Один PR для full-stack features
- ✅ Atomic commits (backend + frontend)
- ✅ Одна версия для всего
- ✅ Проще CI/CD
- ✅ Общая документация
- ✅ Нет проблем с версиями API

**Когда менять на Polyrepo:**
- Команда > 10 человек
- Backend и frontend разрабатываются разными командами
- Backend используется несколькими frontend'ами

**Для Kroma сейчас:** Monorepo идеально! ✅

---

## 📦 Next Steps

1. **Оставить текущую структуру** (уже monorepo!)
2. **Добавить CI/CD workflows** в `.github/workflows/`
3. **Добавить корневой package.json** с общими scripts
4. **Настроить path-based CI triggers**

**Готовы продолжить?**
