# marius-agenda-v2

Agenda scolaire configurable — génération PDF livret, **application desktop Linux native** (GTK, pas de navigateur embarqué type Chromium).

## Cible produit

- Config JSON (`config.json`), calendrier scolaire et **mise en page PDF** livret (page à page ou imposé)
- Parcours UI en **5 étapes** (wizard Libadwaita)
- **Projets** : archive `.marius` (nouveau / ouvrir / récents) ; illustrations et config dans le projet, pas dans le dépôt
- **Binaire natif** : `.deb` / Snap — rendu PDF via **WebKitGTK 6** (comme Epiphany)
- Config par défaut : année scolaire **2026-2027**

## Architecture

| Couche | Techno |
|--------|--------|
| UI desktop | **GTK 4 + Libadwaita** (accueil + wizard natif Linux) |
| Domaine | `agenda-core` (config, calendrier, imposition livret, `.marius`) |
| HTML | `agenda-render` (templates + CSS impression) |
| Images | `agenda-images` (N&B WebP) |
| PDF | `agenda-pdf` — HTML/CSS → **WebKitGTK** (`Print to File`) |

## Installation (utilisateur)

**Snap** (Ubuntu et dérivés, une fois publié sur le canal `stable`) :

```bash
sudo snap install marius-agenda
marius-agenda --gui
```

**Paquet `.deb`** (Ubuntu / Debian amd64) : voir `packaging/debian/README.Debian` (deps GTK/WebKit via `apt`, double-clic `.marius`).

## Développement

Deps système (Ubuntu/Debian) :

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev pkg-config xvfb
```

Tests et exécution locale :

```bash
# Gate complet (unitaires + e2e PDF WebKit + wizard GTK sous xvfb ou DISPLAY)
./scripts/test.sh

# Smoke UI seulement
cargo run -p marius-agenda --features gtk -- --root . --gui-self-test

cargo test

# CLI : lister les périodes
cargo run -p marius-agenda -- --root . periods

# Générer un PDF (session graphique ou xvfb-run)
cargo run -p marius-agenda -- --root . generate --period rentree-premiere-vacances --output ./output

# UI (GTK + Libadwaita) — accueil projets + wizard
cargo run -p marius-agenda --features gtk -- --root . --gui
```

### Packaging local

Paquet `.deb` (après `cargo install cargo-deb`) :

```bash
./scripts/package-deb.sh
# → target/debian/marius-agenda_0.1.0-1_amd64.deb
sudo apt install ./target/debian/marius-agenda_*_amd64.deb
./scripts/verify-deb.sh   # optionnel, VM / machine propre
```

Snap (build + test hors store) :

```bash
sudo snap install snapcraft --classic   # une fois
./scripts/package-snap.sh
# → marius-agenda_0.1.0_amd64.snap
sudo snap install --dangerous ./marius-agenda_*.snap
./scripts/verify-snap.sh   # optionnel
```

Publication Snap Store : `docs/snap-packaging-guide.md`, `docs/snap-store.md`.

Assets : `assets/` (vide — illustrations par projet) et `fonts/` (polices embarquées dans le paquet / snap).

## Roadmap

1. ✅ `agenda-core` + config 2026-2027
2. ✅ `agenda-render` (snapshots HTML)
3. ✅ `agenda-pdf` WebKitGTK
4. ✅ App GTK (accueil `.marius`, wizard 5 étapes, génération par période)
5. ✅ Packaging Snap (`snapcraft.yaml`, scripts verify)
6. 🚧 Packaging `.deb` — validation install sur VM propre (`verify-deb.sh`)

## Licence

[ISC](LICENSE) — voir aussi les métadonnées du workspace dans `Cargo.toml`.
