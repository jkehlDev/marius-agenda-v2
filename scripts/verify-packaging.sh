#!/usr/bin/env bash
# Fast checks for .deb/snap layout (no full release build).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MANIFEST="${ROOT}/apps/marius-agenda/Cargo.toml"
test -f "${MANIFEST}"
grep -q 'package.metadata.deb' "${MANIFEST}"
grep -q 'usr/share/marius-agenda' "${MANIFEST}"
grep -q 'marius-agenda.desktop' "${MANIFEST}"
grep -q 'depends = "\$auto"' "${MANIFEST}"
grep -q 'maintainer-scripts' "${MANIFEST}"
grep -q 'marius-agenda.xml' "${MANIFEST}"
test -f "${ROOT}/apps/marius-agenda/marius-agenda.desktop"
test -f "${ROOT}/packaging/marius-agenda.xml"
test -x "${ROOT}/packaging/debian/scripts/postinst"
test -f "${ROOT}/packaging/debian/README.Debian"
test -f "${ROOT}/packaging/icons/marius-agenda-magic-source.png"
test -f "${ROOT}/packaging/icons/marius-agenda-128.png"
test -d "${ROOT}/assets"
test -d "${ROOT}/fonts"
test -f "${ROOT}/scripts/verify-deb.sh"
test -f "${ROOT}/snap/snapcraft.yaml"
test -f "${ROOT}/snap/gui/marius-agenda.desktop"
test -f "${ROOT}/scripts/package-snap.sh"
test -f "${ROOT}/scripts/verify-snap.sh"

if ! pkg-config --exists webkitgtk-6.0; then
  echo "ERREUR: libwebkitgtk-6.0-dev requis pour le build (apt install libwebkitgtk-6.0-dev)"
  exit 1
fi
echo "packaging metadata OK"
