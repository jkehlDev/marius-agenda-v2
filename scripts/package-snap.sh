#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v snapcraft >/dev/null; then
  echo "Installe snapcraft : sudo snap install snapcraft --classic"
  exit 1
fi

./scripts/verify-packaging.sh

if [[ ! -f packaging/icons/marius-agenda-magic-source.png ]]; then
  echo "ERREUR: packaging/icons/marius-agenda-magic-source.png manquant"
  exit 1
fi
python3 packaging/icons/bake_magic_icon.py
install -Dm644 packaging/icons/marius-agenda-256.png snap/gui/marius-agenda.png

# Optional full clean (avoid `clean marius-agenda` — it can leave an empty part src in LXD).
if [[ "${SNAPCRAFT_CLEAN:-0}" == "1" ]]; then
  echo "== snapcraft clean =="
  snapcraft clean
fi

MODE="${SNAPCRAFT_BUILD_MODE:-lxd}"
echo "== snapcraft pack (mode: ${MODE}) =="
case "${MODE}" in
  lxd)
    snapcraft pack --use-lxd || { echo "ERREUR: snapcraft pack a échoué"; exit 1; }
    ;;
  destructive)
    snapcraft pack --destructive-mode || { echo "ERREUR: snapcraft pack a échoué"; exit 1; }
    ;;
  *)
    echo "SNAPCRAFT_BUILD_MODE invalide (lxd|destructive): ${MODE}"
    exit 1
    ;;
esac

SNAP="$(ls -t "${ROOT}"/marius-agenda_*.snap 2>/dev/null | head -1)"
if [[ -z "${SNAP}" || ! -f "${SNAP}" ]]; then
  echo "ERREUR: aucun fichier .snap trouvé à la racine du dépôt"
  exit 1
fi

if [[ -x "${ROOT}/scripts/verify-snap.sh" ]]; then
  ./scripts/verify-snap.sh "${SNAP}"
fi

echo ""
echo "Snap prêt : ${SNAP}"
echo "Test local  : sudo snap install --dangerous \"${SNAP}\""
echo "Publication : snapcraft login && snapcraft upload --release=stable \"${SNAP}\""
