#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
# Always emit .deb under the repo (not a redirected CARGO_TARGET_DIR).
export CARGO_TARGET_DIR="${ROOT}/target"

if ! command -v cargo-deb >/dev/null; then
  echo "Installe cargo-deb : cargo install cargo-deb"
  exit 1
fi

./scripts/verify-packaging.sh

if [[ ! -f packaging/icons/marius-agenda-magic-source.png ]]; then
  echo "ERREUR: packaging/icons/marius-agenda-magic-source.png manquant"
  exit 1
fi
python3 packaging/icons/bake_magic_icon.py

rm -f "${CARGO_TARGET_DIR}/debian"/marius-agenda_*.deb

cargo deb -p marius-agenda --features gtk

DEB="$(ls -t "${CARGO_TARGET_DIR}/debian"/marius-agenda_*.deb | head -1)"
./scripts/verify-deb.sh "${DEB}"

if command -v lintian >/dev/null; then
  echo "== lintian (avertissements possibles sur paquet non-Debian officiel) =="
  lintian "${DEB}" || true
fi

echo ""
echo "Paquet prêt : ${DEB}"
echo "Installation : sudo apt install ./$(basename "${DEB}")"
