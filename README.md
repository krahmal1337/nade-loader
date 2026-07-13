# nadeloader

> Лаунчер для **Neverlose** под CS:GO.

Инжектит `neverlose.dll` в процесс csgo.exe. Запуск через Steam с флагом `-insecure` — VAC отключён, никаких байпасов не нужно.

---

## Фичи

- **Инжект DLL** — CreateRemoteThread + LoadLibraryA
- **Авто-запуск** игры через Steam Protocol с флагами `-insecure -novid`
- **Ручной режим** — ожидание ручного запуска CS:GO, затем инжект
- **Загрузка с GitHub** — DLL тянется из [krahmal1337/NeverNade](https://github.com/krahmal1337/NeverNade), кэшируется на диск
- **Темы из neverlose.cloud** — встроенные (Blue/Black/Light) + кастомные, MessagePack → CSS
- **Профиль** — аватар с rounded-кропом, имя
- **Выбор конфига** из облака Neverlose
- **Ченджлог** релизов прямо в окне
- **Frameless-окно** — 550×420, прозрачное, drag-to-move
- **Debug консоль** — лог фронта и бэка
- **Авто-закрытие** через 5 сек после удачного инжекта

---

## Быстрый старт

```bash
bun install
bun tauri dev      # dev
bun tauri build    # релиз
```

---

## Структура

```
├── src/                   # SvelteKit — UI
├── src-tauri/             # Rust — бэкенд
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands.rs
│   │   ├── downloader.rs  # GitHub API, загрузка DLL
│   │   ├── steam.rs       # Steam + Win32 инжект
│   │   ├── theme.rs       # state.json, стили MessagePack
│   │   └── error.rs
│   ├── builtin-styles/
│   └── defaults/
├── static/
└── tauri.conf.json
```

---

## Команды

| Tauri-команда | Что делает |
|---|---|
| `load_launcher_theme` | Загружает тему из neverlose.cloud |
| `load_launcher_settings` | Профиль, аватар, конфиги |
| `save_launcher_profile` | Сохраняет имя и аватар |
| `load_git_metadata` | Релизы с GitHub |
| `prepare_version` | Скачивает / кэширует DLL |
| `launch_game_process` | Запускает CS:GO через Steam (`-insecure`) |
| `wait_and_inject` | Ждёт окно и инжектит DLL |
| `kill_background_processes` | Убивает injector.exe |

---

<nl>
  nadeloader · 2026
</nl>
