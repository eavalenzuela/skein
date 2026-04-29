#!/usr/bin/env bash
# Local pre-push smoke gate. Runs every test layer in turn and bails on
# the first failure so feedback is fastest where breakage is cheapest.
#
# Order chosen so cheap checks fail first:
#   1. Type-check (Svelte / TS)
#   2. Lint
#   3. Unit tests (Vitest, ~250 ms)
#   4. Rust tests (~5 s warm cache)
#   5. Playwright e2e (~15 s)
#   6. Tauri integration build — debug, no bundle (~2 min cold, ~30 s warm)
#
# Pass --skip-tauri to drop step 6 when iterating quickly. The other
# steps run in this order regardless.

set -euo pipefail

SKIP_TAURI=0
for arg in "$@"; do
  case "$arg" in
    --skip-tauri) SKIP_TAURI=1 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

step() { printf '\n\033[1;36m▶ %s\033[0m\n' "$*"; }

step "1/6  npm run check"
npm run check

step "2/6  npm run lint"
npm run lint

step "3/6  vitest"
npm run test:unit

step "4/6  cargo test"
cargo test --manifest-path src-tauri/Cargo.toml --tests

step "5/6  playwright"
npm run test:e2e

if [[ $SKIP_TAURI -eq 1 ]]; then
  step "6/6  tauri build — SKIPPED (--skip-tauri)"
else
  step "6/6  tauri build (debug, no bundle)"
  npm run tauri -- build --debug --no-bundle
fi

printf '\n\033[1;32m✓ all smoke checks passed\033[0m\n'
