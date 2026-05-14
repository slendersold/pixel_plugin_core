#!/usr/bin/env bash
# Сборка плагинов не-Rust в каталог артефактов (по умолчанию target/debug).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${TARGET_DIR:-$ROOT/target/debug}"
mkdir -p "$TARGET"

echo "==> libmirror_c.so (C)"
gcc -shared -fPIC -O2 -Wall -Wextra -o "$TARGET/libmirror_c.so" "$ROOT/mirror_c/mirror.c"

echo "==> librotate_go.so (Go c-shared)"
(
  cd "$ROOT/rotate_go"
  go build -buildmode=c-shared -o "$TARGET/librotate_go.so" .
)

echo "Плагины в: $TARGET"
