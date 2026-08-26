default:
    @just --list

# fmt + clippy -D warnings + тесты
check:
    powershell -ExecutionPolicy Bypass -File tools/check.ps1

# release-сборка
release:
    cargo build --release

# перегенерация скриншотов для README (тёмная + светлая, только карточка)
screenshots:
    powershell -ExecutionPolicy Bypass -File tools/screenshots.ps1

# перегенерация иконок из assets/icon.svg (icon.png, app.ico, PNG-лестница)
icons:
    powershell -ExecutionPolicy Bypass -File tools/icons.ps1

# установка/обновление обработчика ссылок (+ открытие настроек Windows)
register:
    powershell -ExecutionPolicy Bypass -File tools/register.ps1

# снять регистрацию и удалить %LOCALAPPDATA%\bropicker
unregister:
    powershell -ExecutionPolicy Bypass -File tools/register.ps1 -Uninstall
