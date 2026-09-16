#!/usr/bin/env bash
# Sanity-check a built .deb (layout, no stale chromium bundle).
set -euo pipefail

DEB="${1:-}"
if [[ -z "${DEB}" || ! -f "${DEB}" ]]; then
  echo "Usage: $0 path/to/marius-agenda_*.deb"
  exit 1
fi

LIST="$(mktemp)"
trap 'rm -f "${LIST}"' EXIT
dpkg-deb -c "${DEB}" >"${LIST}"

grep -q './usr/bin/marius-agenda$' "${LIST}" || {
  echo "ERREUR: binaire usr/bin/marius-agenda absent"
  exit 1
}
grep -q './usr/share/applications/marius-agenda.desktop$' "${LIST}" || {
  echo "ERREUR: fichier .desktop absent"
  exit 1
}
grep -q './usr/share/marius-agenda/assets/' "${LIST}" || {
  echo "ERREUR: assets partagés absents"
  exit 1
}
grep -q './usr/share/icons/hicolor/.*/apps/marius-agenda.png$' "${LIST}" || {
  echo "ERREUR: icône hicolor absente"
  exit 1
}
grep -q './usr/share/mime/packages/marius-agenda.xml$' "${LIST}" || {
  echo "ERREUR: définition MIME absente"
  exit 1
}
if grep -qi chromium "${LIST}"; then
  echo "ERREUR: le paquet contient encore chromium (artefact obsolète)"
  exit 1
fi

CONTROL="$(mktemp)"
trap 'rm -f "${LIST}" "${CONTROL}"' EXIT
dpkg-deb -f "${DEB}" Depends >"${CONTROL}"
grep -qE 'libwebkitgtk-6\.0-' "${CONTROL}" || {
  echo "ERREUR: Depends ne déclare pas libwebkitgtk-6.0-*"
  exit 1
}

echo "verify-deb OK: ${DEB}"
