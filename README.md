# Aura CLI

[![Language](https://img.shields.io/badge/rust-2021-DEA584.svg)](https://www.rust-lang.org)
[![VAC Status](https://img.shields.io/badge/VAC-100%25%20Safe-brightgreen.svg)](#безопасность-и-vac)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-orange.svg)](#сборка-из-исходников)

Легковесная open-source консольная утилита на Rust для базовой и полностью безопасной настройки конфигурации Source 2 в Dota 2.

Aura CLI автоматически находит библиотеку Steam (включая Flatpak/Snap на Linux и реестр на Windows), применяет безопасный базовый профиль графики в `video.txt` и `autoexec.cfg`, и сохраняет изолированный бэкап для каждого Steam-аккаунта.

---

## Что внутри (Community MIT Edition)

- **100% VAC-Safe**: никаких инжектов в память, перехвата вызовов или модификации VPK. Правятся только официальные текстовые конфиги `video.txt` и `autoexec.cfg`.
- **Базовый профиль оптимизации (`basic`)**: отключение ресурсоёмких теней, воды, анимированных портретов, ветра на деревьях, bloom/fog и лишней нагрузки на дашборд + базовый `autoexec.cfg`.
- **Изолированный per-account бэкап и откат**: слепок `video.txt` сохраняется отдельно для каждого `account_id`, а `--restore` аккуратно возвращает исходные файлы или удаляет только блок Aura из `autoexec.cfg`.
- **Поддержка ручных путей**: флаги `--dota-dir` и `--steam-dir` для нестандартных установок.

---

## Community CLI vs Aura Pro

| Возможность | Aura Community CLI (MIT) | Aura Pro |
|---|---|---|
| Базовая настройка `video.txt` и `autoexec.cfg` (`--apply basic`) | ✓ | ✓ |
| Диагностика окружения (`--status`) и откат (`--restore`) | ✓ | ✓ |
| Профили `Competitive`, `Balanced`, `Potato`, `HighRefresh` | — | ✓ |
| Очистка дампов, логов, шейдеров и кэша Panorama | — | ✓ |
| Нейтрализатор Panorama WebM (-1–1.5 ГБ VRAM) | — | ✓ |
| No-Bling (`dota_tempcontent`), Minimap VPK и Steam Launch Studio | — | ✓ |
| Нативный GUI (Hyprland / Windows Mica), профили и автообновление | — | ✓ |

---

## Быстрый старт

### Сборка из исходников

Понадобится компилятор Rust (`cargo`):

```bash
git clone https://github.com/farconiorig/aura-cli.git
cd aura-cli
cargo build --release
```

Бинарный файл появится по пути `target/release/aura-cli` (или `target/release/aura-cli.exe` на Windows). На Linux его можно скопировать в `~/.local/bin/`:

```bash
cp target/release/aura-cli ~/.local/bin/
```

---

## Использование

```bash
# Проверка окружения, аккаунтов и текущего статуса
aura-cli --status

# Применение базового безопасного пресета (video.txt + минимальный autoexec.cfg)
aura-cli --apply basic

# Явное указание нестандартного пути к Dota 2 или Steam
aura-cli --dota-dir "/mnt/games/SteamLibrary/steamapps/common/dota 2 beta" --status

# Откат изменений из бэкапа (по каждому аккаунту отдельно)
aura-cli --restore
```

---

## Безопасность и VAC

Aura CLI не взаимодействует с оперативной памятью запущенной игры и не изменяет исполняемые или VPK-файлы. Правки касаются исключительно стандартных пользовательских конфигов `video.txt` и `autoexec.cfg`.

---

## Лицензия

Проект распространяется под открытой лицензией [MIT](LICENSE).
