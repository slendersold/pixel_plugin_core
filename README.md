# pixel_plugin_core

CLI-приложение для обработки PNG с динамическими плагинами через FFI (`process_image`). Проектная работа модуля 4 (см. ТЗ в репозитории курса).

## Структура

| Путь | Назначение |
|------|------------|
| `Cargo.toml` | Workspace: `image_processor`, `mirror_plugin`, `blur_plugin` |
| `image_processor/` | Бинарник и модули `error`, `plugin_loader` |
| `mirror_plugin/` | Плагин `cdylib` — зеркальное отражение |
| `blur_plugin/` | Плагин `cdylib` — размытие |

## Сборка

```bash
cargo build
```

Плагины попадут в `target/debug/libmirror_plugin.so`, `target/debug/libblur_plugin.so` (на Linux; на Windows — `.dll` без префикса `lib`).

## Пример запуска

```bash
cargo build
echo '{}' > /tmp/params.txt
./target/debug/image_processor \
  input.png output.png mirror_plugin /tmp/params.txt \
  --plugin-path target/debug
```

Аргументы: `input`, `output`, `plugin`, `params`, опционально `--plugin-path` (по умолчанию `target/debug`).

## Зависимости основного крейта

`image`, `clap`, `libloading` — как в чек-листе задания.
