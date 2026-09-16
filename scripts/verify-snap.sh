#!/usr/bin/env bash
set -euo pipefail
SNAP="${1:?usage: verify-snap.sh path/to/marius-agenda_*.snap}"
test -f "${SNAP}"

if ! command -v unsquashfs >/dev/null; then
  echo "skip verify-snap (unsquashfs absent)"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
unsquashfs -q -d "${TMP}/tree" -f "${SNAP}" \
  usr/bin/marius-agenda \
  usr/share/marius-agenda/assets \
  usr/share/marius-agenda/fonts \
  meta/gui/marius-agenda.desktop \
  meta/gui/marius-agenda.png

test -x "${TMP}/tree/usr/bin/marius-agenda"
test -d "${TMP}/tree/usr/share/marius-agenda/assets"
test -d "${TMP}/tree/usr/share/marius-agenda/fonts"
test -f "${TMP}/tree/meta/gui/marius-agenda.desktop"
test -f "${TMP}/tree/meta/gui/marius-agenda.png"
echo "snap layout OK: ${SNAP}"
