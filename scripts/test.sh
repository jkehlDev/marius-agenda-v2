#!/usr/bin/env bash
# Full local verification (same as CI).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "== packaging metadata =="
./scripts/verify-packaging.sh

echo "== cargo test (workspace) =="
run_e2e_pdf() {
  # WebKit may SIGABRT during process teardown after assertions passed; trust test output.
  set +e
  local out
  out="$(mktemp)"
  cargo test -p agenda-pipeline --test e2e_pdf >"$out" 2>&1
  local code=$?
  set -e
  cat "$out"
  if [[ "$code" -eq 0 ]]; then
    return 0
  fi
  if grep -q "e2e_pdf_pipeline_one_and_all_periods ... ok" "$out" \
    && grep -q "test result: ok" "$out"; then
    echo "note: e2e_pdf assertions OK (WebKit teardown exit $code ignoré)"
    return 0
  fi
  return "$code"
}

run_workspace_tests() {
  cargo test -p agenda-core -p agenda-render -p agenda-images -p agenda-pdf
  run_e2e_pdf
}
if command -v xvfb-run >/dev/null; then
  xvfb-run -a bash -c "cd \"$ROOT\" && $(declare -f run_e2e_pdf run_workspace_tests); run_workspace_tests"
elif [[ -n "${DISPLAY:-}" ]]; then
  run_workspace_tests
else
  echo "ERREUR: xvfb ou DISPLAY requis pour les tests PDF WebKit (e2e_pdf)."
  exit 1
fi
if command -v xvfb-run >/dev/null; then
  xvfb-run -a bash -c "cd \"$ROOT\" && cargo test -p marius-agenda --features gtk"
elif [[ -n "${DISPLAY:-}" ]]; then
  cargo test -p marius-agenda --features gtk
else
  echo "ERREUR: xvfb ou DISPLAY requis pour les tests marius-agenda (PDF WebKit)."
  exit 1
fi

echo "== GTK build =="
cargo build -p marius-agenda --features gtk

echo "== GTK GUI self-test =="
run_gui_self_test() {
  cargo run -p marius-agenda --features gtk --quiet -- --root "$ROOT" --gui-self-test
}
if command -v xvfb-run >/dev/null; then
  xvfb-run -a bash -c "cd \"$ROOT\" && cargo run -p marius-agenda --features gtk --quiet -- --root \"$ROOT\" --gui-self-test"
elif [[ -n "${DISPLAY:-}" ]]; then
  run_gui_self_test
else
  echo "ERREUR: installe xvfb (sudo apt install xvfb) ou lance depuis une session graphique (DISPLAY)."
  exit 1
fi

echo "OK"
