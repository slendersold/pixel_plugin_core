# pixel_plugin_core

CLI-приложение для обработки PNG с динамическими плагинами через FFI (`process_image`). Проектная работа модуля 4 (см. ТЗ в репозитории курса).

В ТЗ (§3) для простоты сказано, что плагины считаются написанными на Rust; в этом репозитории **контракт FFI один и тот же** (`process_image`), а демонстрационные реализации на разных языках: размытие — **Rust** (`blur_plugin`), зеркало — **C** (`mirror_c`), поворот — **Go** (`rotate_go`).

## Структура

| Путь | Назначение |
|------|------------|
| `Cargo.toml` | Workspace: `image_processor`, `blur_plugin` (Rust `cdylib`) |
| `image_processor/` | Бинарник и модули `error`, `plugin_loader`, `contract` |
| `blur_plugin/` | Размытие (Rust `cdylib`) |
| `mirror_c/` | Зеркало на **чистом C** → `libmirror_c.so` |
| `rotate_go/` | Поворот 90° на **Go** (`c-shared`) → `librotate_go.so` |
| `scripts/build-plugins.sh` | Сборка C / Go в `target/debug` |
| `scripts/run-integration-sample.sh` | Один вызов: сборка + три прогона CLI как в интеграционном сценарии |

## Сборка Rust

```bash
cargo build --workspace
```

## Сборка всех плагинов (Rust + C + Go)

Нужны `gcc` и `go`. На Ubuntu см. шаги в `.github/workflows/ci-main.yml`.

```bash
cargo build --workspace
bash scripts/build-plugins.sh
```

Артефакты (Linux): `target/debug/libblur_plugin.so`, `libmirror_c.so`, `librotate_go.so`.

## Пример запуска

```bash
cargo build --workspace
bash scripts/build-plugins.sh
echo '{"horizontal":true,"vertical":false}' > /tmp/params.txt
./target/debug/image_processor \
  input.png output.png mirror_c /tmp/params.txt \
  --plugin-path target/debug
```

Аргументы: `input`, `output`, `plugin`, `params`, опционально `--plugin-path` (по умолчанию `target/debug`).

## Интеграционный тест (три плагина)

На **Linux** при `cargo test --workspace` (или `cargo test -p image_processor`) тест `integration_three_plugins` сам вызывает `cargo build -p blur_plugin` и `bash scripts/build-plugins.sh` с нужным `TARGET_DIR`, затем трижды запускает собранный `image_processor`. Нужны **gcc** и **go** в `PATH` (как в CI).

На других ОС этот тест — заглушка.

### Тот же сценарий на своём PNG (артефакты на диске)

Скрипт повторяет параметры из `image_processor/tests/integration_three_plugins.rs`, собирает workspace и внешние `.so` (каталог `debug` берётся из `cargo metadata`, в том числе при заданном `CARGO_TARGET_DIR`) и трижды вызывает `cargo run -p image_processor`. В выходной папке появляются `01_mirror_c.png`, `02_blur_plugin.png`, `03_rotate_go.png` и `params_*.txt`.

```bash
cd pixel_plugin_core
bash scripts/run-integration-sample.sh
```

По умолчанию входной файл — `DachaFriends.png` в корне воркспейса, выход — `integration_outputs/`. Можно указать явно:

```bash
bash scripts/run-integration-sample.sh путь/к/входу.png путь/к/папке_вывода
```

Только прогон без сборки (если артефакты уже в том же `target/.../debug`, что видит Cargo):

```bash
bash scripts/run-integration-sample.sh --no-build
```

Для разбора `cargo metadata` в скрипте нужен `python3` в `PATH`.

## Зависимости основного крейта

`image`, `clap`, `libloading` — как в чек-листе задания.
