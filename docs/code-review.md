# Ревью кода bropicker (2026-08-26) — ВЫПОЛНЕНО

> Скоуп: src/main.rs (401 строка), src/config.rs (~210), src/winit.rs (45),
> ui/*.slint. Плюс вывод clippy.
>
> Итог рефакторинга: main.rs 401→32 строки (чистый скелет), появился launcher.rs
> и app.rs (`struct App`: new/wire×4 группы/run), 12 юнит-тестов,
> tools/check.ps1 (fmt+clippy -D warnings+test) зелёный, дублирование запуска
> устранено, индекс из UI вместо поиска по имени, все P0-P2 закрыты.
> UI проверен скриншотом — идентичен. Cold start release 110-133 ms.

## P0 — мусор и артефакты правок (чинить сразу)

- [ ] `main.rs:141-142` — **дубль** `let url = main_window.get_current_url()..`
      (первый не используется; поймано clippy). Артефакт моих правок этапа D/E
- [ ] `main.rs:229-231` — `on_settings_clicked` зарегистрирован **дважды**:
      мёртвый первый вариант с `println!`. Второй вызов перезаписывает, но мусор
      путает
- [ ] Остатки `println!` (строки 230, 268, 281, 304): в release никуда не пишут,
      в debug сорят. Либо `config::log`, либо удалить
- [ ] Обратная конвертация `BrowserConfig → BrowserEntry` теряет emoji/icon
      (`icon: String::new()` в двух местах). На запуск не влияет, но это бомба:
      любая будущая фича, читающая поля из этой конвертации, получит пустышку

## P1 — дублирование и SRP

- [ ] **Дубль логики запуска**: `on_launch_browser` и `on_open_clicked`
      совпадают на ~90% (лог + launch + remember + hide). Extract:
      `fn launch_and_remember(&Config, &MainWindow, entry, url)`
- [ ] **main.rs — файл-бог** (401 строка): бутстрап + клей UI-моделей + запуск
      браузеров + парсер флагов + иконки + remembered-логика. Разрезать:
      - `launcher.rs` — split_flags, launch, launch_and_remember
      - `config.rs` оставить: TOML + реестр + пути (или вынести `detect.rs`)
      - `main.rs` — тонкий: env, fast-path, show/run (~80 строк)
- [ ] Ручные конвертации моделей ↔ конфиг дважды → `impl From<&BrowserConfig>
      for BrowserEntry` (+ обратный From)
- [ ] Поиск индекса по имени (`find(|b| b.name == browser.name)`): ломается при
      одинаковых именах (два профиля «Chrome»). Правильно: передавать индекс из
      UI — в slint цикл уже знает `i`: `clicked => root.launch-browser(i)`
- [ ] `struct State`: поля browser_model/remembered_model/config после init не
      читаются (Rc живут в замыканиях, модели — в свойствах окон). Оставить
      только окна, или убрать структуру вовсе

## P2 — rust-way (clippy + идиоматика)

- [ ] clippy: `winit.rs:9` match → if let; `load_icon` — collapse nested if let
      (`cargo clippy --fix` закрывает все три)
- [ ] Нет юнит-тестов. Чистые функции напрашиваются:
      - `split_flags`: кавычки с пробелами, пустая строка, двойные пробелы
      - `domain_of`: схема/без схемы/www/порт/путь/userinfo
      - `extract_exe`: кавычки, аргументы, относительный путь
      - `is_known_browser`: регистр, «Яндекс», подстрока в пути-подобном имени
- [ ] В воркфлоу нет `cargo fmt`/`clippy` — добавить `tools/check.ps1`
      (fmt --check + clippy -D warnings + test) и гонять перед коммитом
- [ ] Магические строки → константы: URL-плейсхолдер `"ku6epxboctuk.github.io"`,
      префикс лога `"[bp]"`, имена env (`BP_THEME/BP_CONFIG/BP_VIEW`)
- [ ] `Settings::default` ручной — заменить на `#[derive(Default)]` c
      `#[default = true]`-эквивалентом (serde default-функции уже есть)

## P3 — косметика и Slint

- [ ] `ui/main.slint:1` — неиспользуемые импорты `Button, CheckBox` (CheckRow их
      заменил)
- [ ] Хардкод акцента `#3b82f6` в 4 местах slint → токен `Theme.accent`
- [ ] `theme.slint`: неиспользуемые тени shadow-primary-x/y/blur,
      text-control-muted — проверить и вычистить
- [ ] `settings.slint`: строки браузеров и доменов почти одинаковые — общий
      компонент-строка (низкий приоритет, панелей всего две)
- [ ] Лог без ротации: при желании — усечение до N КБ при старте

## Что трогать НЕ нужно

- `unwrap()` при создании окон в main — норм для startup-фейлов приложения
- `Rc<RefCell<Config>>` — осознанный выбор для однопоточного Slint (не тащить
  Mutex ради галочки)
- Частота `config::save` на каждый чекбокс — частоты ничтожны

## Порядок исполнения (одна сессия)

1. [x] P0 целиком + прогон clippy --fix (P2 первая строка)
2. [x] P1: launcher.rs + launch_and_remember + From-конвертации + индекс из UI
3. [x] Юнит-тесты чистых функций + tools/check.ps1
4. [x] P3 косметика: импорты, Theme.accent/danger токены

Каждый пункт — зелёный `tools\check.ps1` перед коммитом.
