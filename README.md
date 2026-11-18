# bropicker

Перехватывает ссылки `http`/`https` и спрашивает, каким браузером открыть: окно
со списком браузеров, правилами по домену и настройками. Написан на BRUL.

## Соседство с BRUL

Репозиторий приложения и рабочая копия BRUL лежат рядом:

```text
brul/
    bropicker/
```

Зависимость — локальный путь, а не версия: `brul = { path = "../brul" }` в
`[dependencies]` и в `[build-dependencies]` (у `build.rs` своя сборка). Версия
BRUL не пинится и не тегируется: сборка берёт текущее состояние `../brul`,
поэтому ломающее изменение в BRUL ломает и сборку здесь, пока BRUL не поправлен.
CI нет, `cargo` собирает оба репозитория сам.

## Сборка и запуск

```powershell
cargo run
```

Первая сборка заодно собирает BRUL из `../brul`. `just check` — форматирование,
clippy и тесты; `just test` — только тесты.

## Регистрация браузером

```powershell
just register
```

Собирает release, копирует `bropicker.exe` и `assets` в
`%LOCALAPPDATA%\bropicker-brul`, регистрирует обработчик `http`/`https` в `HKCU`
(права администратора не нужны) и открывает настройки Windows: там bropicker
выбирается для `HTTP` и `HTTPS`.

`just unregister` убирает записи реестра и установленный каталог.

## Конфиг

`%APPDATA%\bropicker\config.toml`; другой файл задаётся переменной `BP_CONFIG`.
Пока браузеры не настроены и не найдены в системе, окно показывает
демонстрационный набор.

```toml
theme = "dark"

[[browsers]]
name = "Firefox"
path = "firefox"
default = true

[[browsers]]
name = "Chrome"
path = "chrome"

[rules]
"example.com" = "Chrome"

[settings]
always_ask = false
```

`path` — имя в `PATH` или полный путь к `.exe`; `flags` добавляются к командной
строке. Правило по домену открывает ссылку без окна, если в `[settings]` не
включено `always_ask`.

## Что внутри

| Файл                                                           | Роль                                          |
| -------------------------------------------------------------- | --------------------------------------------- |
| `src/window.brul`                                              | окно, разметка и скрипт                       |
| `src/browser_item.brul`, `check_row.brul`, `settings_row.brul` | компоненты списка и настроек                  |
| `src/main.rs`                                                  | хост, команды окна, разбор аргументов, ассеты |
| `src/config.rs`                                                | разбор TOML, поиск браузеров, правила         |
| `src/launcher.rs`                                              | командная строка запуска и ошибки             |
| `src/generated.rs`                                             | подключение вида, собранного из `window.brul` |
| `tools/register.ps1`                                           | записи реестра и установка                    |
| `assets/`                                                      | иконки и логотипы браузеров                   |

Логотипы браузеров — из набора browser-logos, лицензия рядом с ними:
`assets/logos/LICENSE-browser-logos.txt`.
