# План: удобная разработка bropicker с ИИ-агентом (Slint)

> Статус: принят к исполнению. Дата: 2026-08-25.
>
> Предыстория: эксперимент dioxus-native/Blitz закрыт (прозрачность окна на Windows
> нереализуема без форка blitz-shell/winit на этапе создания HWND). Ветка-архив:
> `dioxus-archive`. Работаем на Slint, основная ветка `main`.

## Принцип «агент видит картинку»

Цикл работы агента над дизайном:

```txt
правка ui/*.slint → сборка+запуск в фоне → скриншот окна → сравнение с ref-PNG → правка
```

Агент читает PNG напрямую (инструмент чтения файлов поддерживает картинки),
поэтому визуальная сверка с макетами происходит без участия человека.

Инструменты визуализации:

- **PS-скрипт скриншота окна** (основной): ноль зависимостей, любая версия slint,
  снимает реальный композитинг — важно для прозрачности поверх стола.
  Минус: окно должно быть видимым
- **`slint-snapshot`** (опционально, позже): headless, детерминированный,
  пиксель-диф из коробки. Минусы: требует ровно `slint =1.17.1` (апгрейд с 1.16.1)
  и возню со шрифтами/embed ресурсов

HTML-макеты → PNG: **headless Edge** (`--headless --screenshot`), он предустановлен на Windows.

---

## Этап A — порядок в репозитории

- [x] Удалена `example/` — копия примера Slint с копирайтом SixtyFPS и битыми path-deps
- [x] Zip `browser-logos` удалён (не был в git; перезаливаемо с github alrrr/browser-logos);
      LICENSE логотипов сохранён в `logos/LICENSE-browser-logos.txt`;
      в `logos/` уже лежат отобранные PNG (chrome, firefox, opera, yandex, brave)
- [x] `.gitignore`: `tools/out/`, `*.log`, `coldstart.txt`

## Этап B — утилиты в `tools/`

Все скрипты — PowerShell, без сторонних зависимостей.

- [ ] `tools/render-refs.ps1` — рендерит `refs/dark.html` и `refs/light.html` через
      headless Edge в `tools/out/ref-dark.png` / `ref-light.png` (окно 488×800,
      `--hide-scrollbars`). Находит msedge.exe в стандартных путях
- [ ] `tools/shot.ps1 [-Title bropicker] [-Out tools/out/app.png]` — скриншот окна
      приложения: FindWindowW + GetWindowRect (Win32 через Add-Type) +
      System.Drawing CopyFromScreen
- [ ] `tools/run-and-shot.ps1` — полный цикл агента: `cargo build` →
      `Start-Process target\debug\bropicker.exe` → пауза ~1.5 c → shot.ps1 →
      `Stop-Process`. Решает проблему «GUI-процесс блокирует терминал агента»
- [ ] Опционально: `slint-lsp` / VS Code extension — ей для редактора, агент не зависит

Проверка цикла: `render-refs` → `run-and-shot` → оба PNG существуют, агент их прочитал.

## Этап C — возвращение к no-frame окну (цель дизайна)

Текущее: окно с заголовком (специально, чтобы двигать мышью). Целевое: frameless +
прозрачность + драг за хедер, как в макете.

- [x] `no-frame: true` + `background: transparent` в MainWindow (+ `title: "bropicker"`)
- [x] Прозрачность подтверждена скриншотами: карточка со скруглёнными углами парит
      над рабочим столом, тёмная и светлая темы
- [x] Драг: TouchArea на хедере (`pointer-event` down) → `request-drag` →
      `with_winit_window(drag_window)` через `i-slint-backend-winit`
- [x] Центрирование окна при старте — `src/winit.rs` восстановлен из истории
      (осторожно: git-редирект PowerShell создаёт UTF-16 — файл перезаписан в UTF-8)
- [x] Esc — закрыть: `FocusScope.key-pressed` (в 1.16 нет `key-event`; обработчик
      обязан вернуть `accept`/`reject` во всех ветках)
- [x] `BP_THEME=light` env-флаг + `run-and-shot.ps1 -Light`
- [x] README обновлён (стек, статус, команды dev-цикла, дорожная карта)

Требует ручной проверки: перетаскивание мышью, Esc, центрирование на активном мониторе.

## Этап D — список браузеров (MVP-ядро)

- [x] ListView + BrowserItem восстановлены в main.slint (список растягивается, карточки 84px)
- [x] Логотипы: PNG из `logos/`; `BrowserConfig.icon` теперь `image` — грузится в Rust
      через `Image::load_from_path` (@image-url не умеет рантайм-пути; serde-derive
      из types.slint убран — вернём в этапе E строкой в TOML)
- [x] Мок-данные 4 браузеров; selected-index = первый is_default
- [x] Выбор кликом + ring-подсветка + hover; галочка — SVG (`icons/check.svg`),
      т.к. глифа U+2713 в шрифтах нет; добавлен токен `Theme.surface-item`
- [x] Чекбоксы «Запомнить выбор…»/«Всегда спрашивать» — кастомный CheckRow по макету
- [x] «Открыть» → `Command::new(path).args(flags).arg(url)`; при успехе окно скрывается
      (loop завершается), при ошибке — сообщение в stderr
- [x] URL из argv (`std::env::args().nth(1)`)

Примечание: мок-пути (firefox.exe, chrome.exe, zen.exe) вне PATH — запуск вернёт
ошибку в stderr до этапа E (автодетект/конфиг). Клик-выбор проверяется визуально.

### Этап D2 — UX-полировка списка

- [x] Отступ списка слева/справа 12px (карточки не прилипают к краям)
- [x] Индикатор выбора без синего круга — синяя SVG-галочка 16px;
      у невыбранных — шеврон «›»
- [x] Клик по карточке = немедленный запуск (`launch-browser(BrowserConfig)`);
      «Открыть» в футере остаётся (запуск выбранного)

## Этап E — конфиг и настройки

- [ ] `config.toml` рядом с exe / `%APPDATA%\bropicker` (крейс `toml`), serde-структура BrowserConfig
- [ ] Логика запуска с учётом галочек: remember-choice → записать выбор для домена;
      always-ask → показывать пикер несмотря на запомненное
- [ ] Страница настроек: TabWidget + LineEdit/SpinBox/ComboBox/CheckBox std-widgets;
      вход по кнопке cog из футера (колбэк settings-clicked уже есть)
- [ ] Автодетект установленных браузеров (реестр Windows) как fallback при отсутствии конфига
- [ ] Чтение флагов из .lnk ярлыков (крейс `lnk`) — по желанию

## Этап F — релиз

- [ ] `[profile.release]`: lto, codegen-units=1, strip
- [ ] Замер cold start: Instant в начале main → печать в первом колбэке; цель <100 ms;
      результат записать сюда: `___ ms`
- [ ] Иконка exe (.ico через build.rs winres)
- [ ] Выбрать лицензию проекта: GPLv3 (опенсорс) или Royalty-free Slint (закрытый десктоп,
      атрибуция AboutSlint) — зафиксировать в README и LICENSE
- [ ] Скриншоты в README из tools/out

## Заметки для агента

- GUI-приложение запускать только через `tools/run-and-shot.ps1`, никогда не блокирующе
- После каждого изменения UI: скриншот + сравнение с `tools/out/ref-{dark,light}.png`
- Тёмная/светлая тема переключается кнопкой sun/moon (Theme.toggle уже работает);
  для скриншота светлой темы кликать нельзя — добавить env-флаг `BP_THEME=light`,
  который в main() выставит `Theme.is-dark = false` перед показом
- Cargo.toml правится только через cargo CLI; манифесты руками — только человеком
