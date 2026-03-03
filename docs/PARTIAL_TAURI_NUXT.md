# Tauri Частичный (Partial Tauri) Архитектура

**Цель:** Использовать Tauri только там, где он нужен, остальное — как обычное веб-приложение.

---

## 📋 Что такое "Частичный Tauri"?

**Идея:** Приложение работает **и в Tauri, и в браузере**, используя Tauri API только для специфичных функций:

```
┌─────────────────────────────────────────────────────────────┐
│                    Nuxt Frontend (Vue 3)                    │
│                   (одинаковый код для обоих режимов)         │
├─────────────────────────────────────────────────────────────┤
│  Функции                    │  Режим                       │
│  ───────────────────────────┼────────────────────────────  │
│  ✅ Роутинг                  │  Веб (Vue Router)            │
│  ✅ API запросы              │  Веб (fetch/axios)           │
│  ✅ UI компоненты            │  Веб (Nuxt components)       │
│  ⚡ Файловые диалоги         │  Tauri (только в desktop)    │
│  ⚡ Системный трей           │  Tauri (только в desktop)    │
│  ⚡ Авто-обновления          │  Tauri (только в desktop)    │
│  ⚡ Native notifications     │  Tauri (только в desktop)    │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎯 Два Режима Работы

### Режим 1: Tauri Desktop

```
┌──────────────────────────────────────────────────────────────┐
│  Tauri Desktop App                                           │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Nuxt Frontend (Vue 3)                                 │  │
│  │  - Работает внутри Tauri WebView                       │  │
│  │  - API: http://localhost:8788                          │  │
│  │  - Tauri API доступны через @tauri-apps/api            │  │
│  └────────────────────────────────────────────────────────┘  │
│                              ↓                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Rust Backend (встроенный)                             │  │
│  │  - Запускается как background process                  │  │
│  │  - SQLite + локальные файлы                            │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Режим 2: Web Browser

```
┌──────────────────────────────────────────────────────────────┐
│  Web Browser (Chrome, Firefox, Safari)                       │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Nuxt Frontend (Vue 3) - ТОТ ЖЕ КОД!                   │  │
│  │  - Работает в браузере                                 │  │
│  │  - API: https://api.kroma.app                          │  │
│  │  - Tauri API НЕ доступны (fallback на веб API)         │  │
│  └────────────────────────────────────────────────────────┘  │
│                              ↓                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Rust Backend (на сервере)                             │  │
│  │  - PostgreSQL + S3                                     │  │
│  │  - Multi-user + JWT auth                               │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔧 Ключевые Принципы

### 1. Tauri API Только Там, Где Нужно

```typescript
// ✅ ПРАВИЛЬНО: Tauri только для файловых диалогов

// composables/useFileDialog.ts
import { open } from '@tauri-apps/api/dialog'

export function useFileDialog() {
  const selectFile = async () => {
    // Проверяем, запущены ли в Tauri
    if (window.__TAURI__) {
      // Tauri mode: нативный диалог
      const path = await open({
        multiple: false,
        filters: [{
          name: 'Images',
          extensions: ['png', 'jpg', 'jpeg', 'webp']
        }]
      })
      return path // Возвращаем путь к файлу
    } else {
      // Web mode: стандартный input
      return null // fallback на <input type="file">
    }
  }
  
  return { selectFile }
}
```

```typescript
// ❌ НЕПРАВИЛЬНО: Зависимость от Tauri везде

// Не делайте так!
const path = await open({ ... }) // Сломается в браузере!
```

---

### 2. Адаптер для Файлов

```typescript
// composables/useFileAdapter.ts

export function useFileAdapter() {
  const isTauri = !!window.__TAURI__
  
  // Загрузка файла
  const uploadFile = async (file: File | string) => {
    if (isTauri && typeof file === 'string') {
      // Tauri mode: отправляем путь
      return await $fetch('/api/upload', {
        method: 'POST',
        body: { file_path: file }
      })
    } else if (file instanceof File) {
      // Web mode: отправляем FormData
      const formData = new FormData()
      formData.append('file', file)
      
      return await $fetch('/api/upload', {
        method: 'POST',
        body: formData
      })
    }
  }
  
  // Скачивание файла
  const downloadFile = async (path: string) => {
    if (isTauri) {
      // Tauri mode: сохраняем на диск
      const { save } = await import('@tauri-apps/api/dialog')
      const filePath = await save({ defaultPath: 'image.png' })
      
      if (filePath) {
        const { writeBinaryFile } = await import('@tauri-apps/api/fs')
        const blob = await fetch(path).then(r => r.blob())
        const bytes = await blob.arrayBuffer()
        await writeBinaryFile(filePath, new Uint8Array(bytes))
      }
    } else {
      // Web mode: стандартное скачивание
      const a = document.createElement('a')
      a.href = path
      a.download = 'image.png'
      a.click()
    }
  }
  
  return { uploadFile, downloadFile }
}
```

---

### 3. Авто-определение Режима

```typescript
// composables/useRuntimeMode.ts

export function useRuntimeMode() {
  const isTauri = !!window.__TAURI__
  
  const config = {
    apiUrl: isTauri 
      ? 'http://localhost:8788'  // Desktop mode
      : 'https://api.kroma.app', // Web mode
    
    authRequired: !isTauri,       // Web требует auth
    fileMode: isTauri ? 'path' : 'formData',
    offlineSupport: isTauri,      // Только desktop offline
  }
  
  return { isTauri, config }
}
```

---

## 📁 Структура Проекта Nuxt + Tauri

```
app/
├── src-tauri/                    # Rust backend (существующий)
│   ├── src/
│   │   ├── api/                  # API routes
│   │   ├── db/                   # Database layer
│   │   └── main.rs               # Entry point
│   └── Cargo.toml
│
├── frontend/                     # Nuxt Frontend (NEW)
│   ├── src-tauri/                # Tauri Desktop App
│   │   ├── src/
│   │   │   ├── main.rs           # Tauri setup
│   │   │   └── commands.rs       # Tauri commands
│   │   ├── tauri.conf.json       # Tauri config
│   │   ├── Cargo.toml
│   │   └── build.rs
│   │
│   ├── nuxt.config.ts            # Nuxt config
│   ├── package.json
│   │
│   ├── composables/              # Vue composables
│   │   ├── useRuntimeMode.ts     # Desktop vs Web detection
│   │   ├── useFileAdapter.ts     # File upload adapter
│   │   ├── useFileDialog.ts      # File dialog adapter
│   │   └── useAutoUpdate.ts      # Tauri auto-update
│   │
│   ├── components/               # UI components
│   │   ├── FilePicker.vue        # Универсальный picker
│   │   ├── ProjectCard.vue       # Карточка проекта
│   │   └── ...
│   │
│   ├── pages/                    # Nuxt pages (J00-J08)
│   │   ├── index.vue             # J00: Онбординг
│   │   ├── projects.vue          # J01: Проекты
│   │   ├── references.vue        # J02: Референсы
│   │   └── ...
│   │
│   └── plugins/
│       └── tauri.client.ts       # Tauri plugin (client-side only)
│
└── docs/
```

---

## 🚀 Как Это Работает

### Шаг 1: Nuxt App в Tauri WebView

```rust
// frontend/src-tauri/src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::SocketAddr;
use kroma_backend_core::api::server::serve;

#[tokio::main]
async fn main() {
    // Запускаем Rust backend в background
    let backend_addr: SocketAddr = "127.0.0.1:8788".parse().unwrap();
    
    tokio::spawn(async move {
        serve(backend_addr).await.unwrap();
    });
    
    // Запускаем Tauri WebView с Nuxt frontend
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Шаг 2: Nuxt Запускается в WebView

```vue
<!-- frontend/pages/index.vue -->

<template>
  <div>
    <h1>Kroma</h1>
    
    <!-- Кнопка выбора файла -->
    <FilePicker @select="handleFileSelect" />
    
    <!-- Отображение режима -->
    <div v-if="isTauri">
      🖥️ Desktop Mode (Tauri)
    </div>
    <div v-else>
      🌐 Web Mode (Browser)
    </div>
  </div>
</template>

<script setup>
const { isTauri, config } = useRuntimeMode()
const { uploadFile } = useFileAdapter()

const handleFileSelect = async (file) => {
  // Работает и в Tauri, и в Web!
  const result = await uploadFile(file)
  console.log('Uploaded:', result)
}
</script>
```

---

### Шаг 3: Tauri Plugins (Client-Side Only)

```typescript
// frontend/plugins/tauri.client.ts

export default defineNuxtPlugin(() => {
  const isTauri = !!window.__TAURI__
  
  if (isTauri) {
    // Импортируем Tauri API только в desktop режиме
    import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
      const win = getCurrentWindow()
      
      // Показываем окно после загрузки
      win.show()
      win.setFocus()
    })
    
    // Авто-обновления
    import('@tauri-apps/plugin-updater').then(({ check }) => {
      check().then(update => {
        if (update) {
          console.log('Update available:', update.version)
          // Показать пользователю диалог обновления
        }
      })
    })
  }
})
```

---

## ⚡ Tauri Функции (Используем Избирательно)

### ✅ Использовать Tauri Для:

| Функция | Tauri API | Web Fallback |
|---------|-----------|--------------|
| **Файловые диалоги** | `@tauri-apps/api/dialog` | `<input type="file">` |
| **Системный трей** | `@tauri-apps/api/tray` | Нет (только desktop) |
| **Авто-обновления** | `@tauri-apps/plugin-updater` | Нет (только desktop) |
| **Native уведомления** | `@tauri-apps/api/notification` | Web Notifications API |
| **Доступ к ФС** | `@tauri-apps/api/fs` | Нет (только download) |
| **Menu bar** | `@tauri-apps/api/menu` | Нет (только desktop) |

### ❌ Не Использовать Tauri Для:

- Роутинг (Vue Router)
- API запросы (fetch/axios)
- UI компоненты (Nuxt components)
- State management (Pinia)
- Формы и валидация

---

## 🔐 Безопасность

### Tauri Security Config

```json
// frontend/src-tauri/tauri.conf.json

{
  "tauri": {
    "security": {
      "csp": "default-src 'self'; connect-src 'self' http://localhost:8788"
    },
    "allowlist": {
      "dialog": {
        "open": true,
        "save": true
      },
      "fs": {
        "scope": ["$HOME/Kroma/**"],
        "read": true,
        "write": true
      },
      "notification": {
        "all": true
      }
    }
  }
}
```

---

## 📦 Build Process

### Desktop Build (Tauri)

```bash
cd frontend

# Build Nuxt app
npm run build

# Build Tauri desktop app
npm run tauri build

# Результат:
# - Windows: .msi, .exe
# - macOS: .app, .dmg
# - Linux: .deb, .AppImage
```

### Web Build

```bash
cd frontend

# Build Nuxt app для веба
npm run build

# Результат:
# - .output/ папка с SSR/SSG
# - Deploy на Vercel/Netlify/VPS
```

---

## 🎯 Преимущества Этого Подхода

| Преимущество | Описание |
|-------------|----------|
| ✅ **Одна кодовая база** | Frontend один для desktop и web |
| ✅ **Быстрый старт** | Nuxt готов из коробки (SSR, routing, etc.) |
| ✅ **Tauri только где нужен** | Не тащим зависимости в web |
| ✅ **Легкий web deploy** | 2-3 недели на адаптацию |
| ✅ **Native UX в desktop** | Файловые диалоги, трей, уведомления |
| ✅ **Backend без изменений** | Rust код одинаковый для обоих |

---

## ⚠️ Потенциальные Проблемы

### 1. Tauri API в Web

**Проблема:** Код с `import { open } from '@tauri-apps/api/dialog'` сломается в браузере.

**Решение:**
```typescript
// composables/useFileDialog.ts
export async function selectFile() {
  if (window.__TAURI__) {
    const { open } = await import('@tauri-apps/api/dialog')
    return await open({ ... })
  }
  return null // fallback
}
```

---

### 2. Пути к Файлам

**Desktop:** `var/projects/my-comic/outputs/image.png`

**Web:** S3 URL `https://bucket.s3.amazonaws.com/...`

**Решение:**
```typescript
// composables/useStorageUrl.ts
export function toDisplayUrl(path: string) {
  const { isTauri } = useRuntimeMode()
  
  if (isTauri) {
    return `file://${path}` // или data URL
  } else {
    return `https://cdn.kroma.app/${path}`
  }
}
```

---

### 3. CORS

**Desktop:** Нет CORS (localhost → localhost)

**Web:** CORS требуется

**Решение:** Добавить CORS middleware в Rust backend:

```rust
// src-tauri/src/api/mod.rs

use tower_http::cors::{CorsLayer, Any};

pub fn create_router() -> Router {
    Router::new()
        // ... routes
        .layer(CorsLayer::new()
            .allow_origin(Any) // Настроить для production!
            .allow_methods(Any)
            .allow_headers(Any)
        )
}
```

---

## 📊 Сравнение Подходов

| Функция | Полный Tauri | Частичный Tauri ✅ | Чистый Web |
|---------|-------------|-------------------|------------|
| **Файловые диалоги** | ✅ Native | ✅ Native (desktop) | ⚠️ Browser |
| **Системный трей** | ✅ Да | ✅ Да | ❌ Нет |
| **Авто-обновления** | ✅ Да | ✅ Да | ❌ Нет |
| **Offline** | ✅ Полный | ✅ Полный | ⚠️ PWA |
| **Web Deploy** | ❌ Нет | ✅ Да | ✅ Да |
| **Код共享** | ❌ 2 кодовых базы | ✅ 1 кодовая база | ✅ 1 кодовая база |

---

## 🎯 Рекомендация

**Используйте Частичный Tauri подход:**

1. **Nuxt frontend** — один для desktop и web
2. **Tauri API** — только для файловых диалогов, трея, обновлений
3. **Адаптеры** — для file upload/download
4. **Авто-определение** — режим определяется автоматически

**Результат:**
- Desktop app с native UX
- Web app с минимальными изменениями
- Одна кодовая база для поддержки

---

## 📦 Следующие Шаги

1. **Создать Nuxt проект** в `frontend/`
2. **Настроить Tauri 2.x** в `frontend/src-tauri/`
3. **Добавить адаптеры** (useFileAdapter, useFileDialog)
4. **Начать с J00-J01** (онбординг + проекты)

**Готовы начать?**
