#!/usr/bin/env bash
# Три прогона CLI (mirror_c → blur_plugin → rotate_go) с теми же JSON-параметрами,
# что в `image_processor/tests/integration_three_plugins.rs`. Результаты — в каталог
# по умолчанию `integration_outputs/` в корне воркспейса.
#
# Запуск из каталога pixel_plugin_core:
#   bash scripts/run-integration-sample.sh
#   bash scripts/run-integration-sample.sh path/to/input.png path/to/out_dir
#   bash scripts/run-integration-sample.sh --no-build   # только прогон, без сборки
#
# Плагины ищутся в `$(cargo metadata …)/debug`, чтобы совпадало с CARGO_TARGET_DIR.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NO_BUILD=0

usage() {
  cat <<'EOF'
Usage: run-integration-sample.sh [--no-build] [INPUT.png] [OUTPUT_DIR]

  Собирает workspace и внешние .so (если не передан --no-build), затем три раза
  запускает image_processor на одном входном PNG (как интеграционный сценарий).

  INPUT.png   по умолчанию: <корень воркспейса>/DachaFriends.png
  OUTPUT_DIR  по умолчанию: <корень воркспейса>/integration_outputs/

  В OUTPUT_DIR: 01_mirror_c.png, 02_blur_plugin.png, 03_rotate_go.png и params_*.txt

  --no-build  пропустить `cargo build --workspace` и scripts/build-plugins.sh
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-build) NO_BUILD=1; shift ;;
    -h | --help) usage; exit 0 ;;
    *) break ;;
  esac
done

INPUT="${1:-$ROOT/DachaFriends.png}"
OUT="${2:-$ROOT/integration_outputs}"

resolve_plugin_dir() {
  local td
  td="$(cd "$ROOT" && cargo metadata --format-version=1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
  echo "${td}/debug"
}

if [[ "$NO_BUILD" -eq 0 ]]; then
  (cd "$ROOT" && cargo build --workspace)
  PLUGIN_DIR="$(resolve_plugin_dir)"
  TARGET_DIR="$PLUGIN_DIR" bash "$ROOT/scripts/build-plugins.sh"
else
  PLUGIN_DIR="$(resolve_plugin_dir)"
fi

for lib in libmirror_c.so libblur_plugin.so librotate_go.so; do
  if [[ ! -f "$PLUGIN_DIR/$lib" ]]; then
    echo "Нет файла $PLUGIN_DIR/$lib — выполните сборку без --no-build." >&2
    exit 1
  fi
done

if [[ ! -f "$INPUT" ]]; then
  echo "Входной PNG не найден: $INPUT" >&2
  exit 1
fi

mkdir -p "$OUT"
printf '%s\n' '{"horizontal":true,"vertical":false}' >"$OUT/params_mirror_c.txt"
printf '%s\n' '{"radius":1,"iterations":1}' >"$OUT/params_blur_plugin.txt"
printf '%s\n' '{"clockwise":true}' >"$OUT/params_rotate_go.txt"

run_one() {
  local dest="$1" plugin="$2" params="$3"
  (cd "$ROOT" && cargo run -p image_processor --quiet -- "$INPUT" "$dest" "$plugin" "$params" --plugin-path "$PLUGIN_DIR")
}

run_one "$OUT/01_mirror_c.png" mirror_c "$OUT/params_mirror_c.txt"
run_one "$OUT/02_blur_plugin.png" blur_plugin "$OUT/params_blur_plugin.txt"
run_one "$OUT/03_rotate_go.png" rotate_go "$OUT/params_rotate_go.txt"

echo "Готово: $OUT"
ls -la "$OUT"
