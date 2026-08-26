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

### E1 — конфиг, автодетект, логика запуска — ГОТОВО

- [x] `src/config.rs`: `%APPDATA%\bropicker\config.toml` (крейсы `toml` + `winreg`);
      env `BP_CONFIG` для тестов; секции `[[browsers]]`, `[remembered]` (домен→имя),
      `[settings]` (remember_choice, always_ask)
- [x] Автодетект из реестра `HKLM/HKCU\SOFTWARE\Clients\StartMenuInternet`:
      имя + exe из shell\open\command, дедуп по пути, проверка существования файла;
      результат автосохраняется. Проверено: найдены Firefox, Zen, Chrome, Edge
- [x] Иконки по имени (firefox/chrome/opera/brave/yandex → `logos/*.png`,
      остальное → globe.svg). Нет логотипов: Edge, Zen
- [x] Логика запуска: домен запомнен + always_ask=off → мгновенный запуск без окна
      (протестировано); иначе пикер. Клик/«Открыть» + remember_choice → запись
      домена в конфиг; переключение галочек персистится
- [x] URL-бар привязан к свойству (был захардкожен), метка чекбокса — домен

### E2 — страница настроек — ГОТОВО (отдельное окно)

- [x] Настройки = **отдельное окно** `SettingsWindow` (760×560, обычная рамка,
      resizable; в маленьком пикере редактировать неудобно). То же событийное
      кольцо; Theme-глобал красит оба окна; модели шарятся (ModelRc один на оба)
- [x] cog в пикере → показать окно (центрирование через invoke_from_event_loop —
      winit-окно реализуется лениво, как и у пикера)
- [x] Две панели: браузеры (иконка/имя/путь/корзина) + запомненные домены
      (domain → browser/корзина); кнопка «Автодетект» (мерж по пути)
- [x] Esc в настройках = закрыть окно; X работает по умолчанию
- [x] env `BP_VIEW=settings` — запуск сразу с окном настроек (без пикера);
      нюанс: `run()` авто-показывает своё окно, поэтому в этом режиме цикл
      крутится на settings_window
- [ ] Редактирование браузера (имя/путь/флаги через LineEdit) — ОТЛОЖЕНО (решено:
      cog в пикере открывает `config.toml` в редакторе — `subl`, fallback `notepad`;
      если файла нет — создаётся с автодетектом). Окно SettingsWindow осталось
      в коде (недоступно из UI), dev-доступ: env `BP_VIEW=settings`
- [ ] Чтение флагов из .lnk ярлыков (крейс `lnk`) — по желанию
- [ ] Добавить логотипы Edge/Zen в logos/ (пересоздать набор из alrrr/browser-logos)

### E3 — доведение до ежедневного использования — ГОТОВО

- [x] cog → открыть конфиг в редакторе (subl → notepad)
- [x] Замер cold start: Instant в main → печать при старте event loop
- [x] `tools/register.ps1` (+ `-Uninstall`): **ставит пакет в
      `%LOCALAPPDATA%\bropicker\` (exe + logos + icons)** и регистрирует
      ProgId/Capabilities/RegisteredApplications в HKCU на путь из LOCALAPPDATA
      (не target\ — переживает cargo clean и перенос репо), открывает
      ms-settings:defaultapps
- [x] Иконки резолвятся от папки exe (иначе при запуске обработчиком ссылок
      cwd = system32 и иконки бы отвалились); проверено запуском из
      LOCALAPPDATA с cwd=System32
- [x] Release-профиль: lto + codegen-units=1 + strip

## Этап F — релиз

- [x] `[profile.release]`: lto, codegen-units=1, strip
- [x] Замер cold start (release, до старта event loop): **183 ms холодный /
      120-122 ms тёплый**; из LOCALAPPDATA первый запуск 153 ms.
      Цель <100 ms чуть не достигнута; варианты оптимизации, если захочется:
      renderer-femtovg вместо skia (быстрее init GPU), отложенная загрузка
      конфига. Пока считаем приемлемым
- [ ] Иконка exe (.ico через build.rs winres)
- [ ] Выбрать лицензию проекта: GPLv3 (опенсорс) или Royalty-free Slint (закрытый десктоп,
      атрибуция AboutSlint) — зафиксировать в README и LICENSE
- [ ] Скриншоты в README из tools/out
- [ ] Ручной сценарий приёмки: register → клик ссылки в другом приложении →
      пикер → выбор → вкладка открылась; повторный клик того же домена при
      always_ask=off → сразу браузер

## Этап F — релиз

- [ ] `[profile.release]`: lto, codegen-units=1, strip
- [ ] Замер cold start: Instant в начале main → печать в первом колбэке; цель <100 ms;
      результат записать сюда: `___ ms`
- [ ] Иконка exe (.ico через build.rs winres)
- [ ] Выбрать лицензию проекта: GPLv3 (опенсорс) или Royalty-free Slint (закрытый десктоп,
      атрибуция AboutSlint) — зафиксировать в README и LICENSE
- [ ] Скриншоты в README из tools/out

## Решение по архитектуре запуска (исследование 2026-08-25)

ОС вызывает пикер так: `CreateProcess("bropicker.exe", "https://...")` — новый процесс
на каждый клик, постоянной связи с вызывающим приложением нет. Два режима:

- **Новый инстанс (выбрано для MVP):** нулевая сложность; если cold start < 100 ms,
  разница с резидентом невидима
- **Резидент + трей (этап G, по показаниям):** spawn-заглушка + IPC + показ готового
  окна (~5-10ms), но: постоянная память ~30-80MB, named mutex, IPC (named pipe /
  WM_COPYDATA), крейс `tray-icon` + скрытое winit-окно (в Slint трея нет), автозапуск

Критерий старта этапа G: замер этапа F показал cold start > 100 ms, либо нужен
доступ к настройкам/функциям из трея. Single-instance guard (named mutex + фокус
существующего окна) можно добавить отдельно, без полного резидента — если быстрые
клики по нескольким ссылкам начнут плодить окна.

## Этап G — резидент + трей (отложен до результатов этапа F)

- [ ] Решение по итогам замера cold start (см. блок выше)
- [ ] Single-instance guard: named mutex, форвард URL в существующий инстанс (named pipe)
- [ ] Иконка в трее (`tray-icon`), меню: Открыть настройки / Выход
- [ ] Автозапуск (HKCU Run или Startup-ярлык)

## Заметки для агента

- GUI-приложение запускать только через `tools/run-and-shot.ps1`, никогда не блокирующе
- После каждого изменения UI: скриншот + сравнение с `tools/out/ref-{dark,light}.png`
- Тёмная/светлая тема: переключение кнопкой sun/moon (Theme.toggle); для скриншота
  светлой темы — `tools/run-and-shot.ps1 -Light` (env `BP_THEME=light`)
- Cargo.toml правится только через cargo CLI; манифесты руками — только человеком
