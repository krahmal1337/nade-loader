# nadeloader reloaded

> Десктопный загрузчик для **NeverLose** под CS:GO / CS2 Legacy.

Без лишнего шума. Тёмная тема, прозрачное окно, горячая перезагрузка стилей из neverlose.cloud, инжект прямо в процесс — всё, что надо.

![Tauri](https://img.shields.io/badge/Tauri-2-ffc131?logo=tauri)
![Svelte](https://img.shields.io/badge/Svelte-5-ff3e00?logo=svelte)
![Rust](https://img.shields.io/badge/Rust-1.85+-dea584?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## Возможности

###  Загрузка и инжект

- **Автоматическая загрузка** `neverlose.dll` с GitHub-релизов ([krahmal1337/NeverNade](https://github.com/krahmal1337/NeverNade)).
- **Ручной режим** — флажок *manual launch*: лаунчер ждёт, пока пользователь сам откроет CS:GO, и инжектит DLL.
- **Авто-запуск** через Steam Protocol (стим запускает игру с флагами `-insecure -novid`).
- **Инжект через `CreateRemoteThread`** + **восстановление `NtOpenFile`** для обхода EAC (EasyAntiCheat).
- **Принудительное завершение** `csgo.exe` перед запуском, если он уже открыт.

###  Стилизация из neverlose.cloud

Лаунчер читает `state.json` и стили из папки `C:\nevernade\cloud`:
- Декодирует **MessagePack**-стили (base64 → rmpv → RGBA).
- Поддерживает **встроенные стили**: Blue, Black, Light.
- Подгружает **кастомные стили** из облака NeverLose.
- Все CSS-переменные (`--nl-text`, `--nl-button`, `--nl-main-bg` и т.д.) автоматически подставляются в UI.

###  Управление профилем

- Аватар с кастомным **rounded-кропом** (256×256, PNG, радиус скругления).
- Имя профиля — сохраняется в `state.json`.
- Поддержка **графических форматов**: PNG, JPEG, GIF, WebP.
- **Work-in-progress** — раздел профиля помечен как черновик.

###  Выбор билда и версии

- **Release / Nightly** — разделение стабильных и тестовых сборок.
- **Выбор конфига** из облачных конфигураций NeverLose.
- **Просмотр ченджлога** — changelog релиза прямо в лаунчере.
- **Кэширование на диск** — уже скачанные DLL не загружаются повторно.

###  UI / UX

- **Frameless-окно** (550×420) с собственным заголовком и контролами.
- **Прозрачный фон** — окно рисуется поверх десктопа.
- **Анимации** — плавный вход панелей, спиннер загрузки, progress bar инжекта.
- **Drag to move** — перетаскивание окна за любую неинтерактивную область.
- **Debug Console** — лог событий фронтенда и бэкенда (встроенная консоль по кнопке `$_`).
- **Блокировка DevTools** — F12/Ctrl+Shift+I перехватываются.

###  Системные

- **Автоопределение установки Steam** через реестр Windows.
- **Ручное указание Steam-пути** для систем без реестра.
- **`taskkill` инжектора** перед загрузкой новой версии.
- **Graceful close** — лаунчер сам закрывается через 5 секунд после успешного инжекта.
- **Минимизация / закрытие** через Tauri API.

---

## Быстрый старт

```bash
git clone https://github.com/krahmal1337/nadeloader
cd nadeloader
bun install
bun tauri dev
```

Для сборки:

```bash
bun tauri build
```

Готовый инсталлятор появится в `src-tauri/target/release/bundle/`.

---

## Структура проекта

```
├── src/                         # SvelteKit — фронтенд
│   ├── routes/+page.svelte      # главный экран (весь UI)
│   └── app.css                  # Tailwind + глобальные стили
├── src-tauri/                   # Rust — бэкенд
│   ├── src/
│   │   ├── main.rs              # точка входа
│   │   ├── lib.rs               # регистрация команд Tauri
│   │   ├── commands.rs          # Tauri-команды
│   │   ├── downloader.rs        # GitHub API, загрузка DLL, сканирование
│   │   ├── steam.rs             # работа со Steam, инжект, Win32 API
│   │   ├── theme.rs             # чтение state.json, MessagePack-стили
│   │   └── error.rs             # типы ошибок
│   ├── builtin-styles/          # Blue.style, Black.style, Light.style
│   ├── defaults/                # state.json по умолчанию
│   └── tauri.conf.json          # конфиг окна и сборки
├── static/                      # статика (иконка игры, шрифты)
├── package.json
└── vite.config.ts
```

---

## Команды Tauri

| Команда | Назначение |
|---|---|
| `load_launcher_theme` | Загружает стиль из neverlose.cloud |
| `load_launcher_settings` | Читает профиль, аватар, список конфигов |
| `save_launcher_profile` | Сохраняет имя и аватар профиля |
| `load_git_metadata` | Получает список релизов с GitHub |
| `prepare_version` | Скачивает / проверяет DLL выбранной версии |
| `launch_game_process` | Запускает игру через Steam Protocol |
| `wait_and_inject` | Ждёт окно CS:GO и инжектит DLL |
| `minimize_main_window` | Сворачивает окно |
| `close_main_window` | Закрывает окно |
| `kill_background_processes` | Убивает `injector.exe` |

---

## Сборка из исходников

**Требования:**
- Node.js 18+ / Bun
- Rust 1.85+
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (WebView2, MSVC)

```bash
bun install                     # зависимости фронтенда
bun tauri dev                   # режим разработки
bun tauri build                 # production-сборка
```

---

## Благодарности

- [NeverLose](https://neverlose.cc) — стили и экосистема
- [Tauri](https://v2.tauri.app) — десктопный рантайм
- [Svelte](https://svelte.dev) — реактивный UI
- Сообществу **nademafia** за поддержку

---

<div align="center">
  <sub>nadeloader · reloaded · 2026</sub>
</div>
