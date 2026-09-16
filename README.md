# marius-agenda-v2

Agenda scolaire configurable — génération PDF livret, **Rust natif** (sans Electron / sans Tauri).

## Cible produit

- Mêmes données et même rendu que [marius-agenda](https://github.com/jkehlDev/marius-agenda) (fidélité maximale)
- Parcours UI en **étapes** (équivalent wizard React v1)
- **Binaire natif** : `.deb` / `.snap` — dépendance **WebKitGTK 6** (comme Epiphany), pas de Chromium embarqué
- Config par défaut : année scolaire **2026-2027** (dates alignées sur la v1)

## Architecture

| Couche | Techno |
|--------|--------|
| UI desktop | **GTK 4 + Libadwaita** (wizard natif Linux) |
| Domaine | `agenda-core` (config, calendrier, imposition livret) |
| HTML | `agenda-render` (port des templates + CSS v1) |
| Images | `agenda-images` (N&B WebP, équivalent Sharp) |
| PDF | `agenda-pdf` — HTML/CSS → **WebKitGTK** (`Print to File`) |

## Développement

Deps système (Ubuntu/Debian) :

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev pkg-config xvfb
```

Vérification complète (unitaires + e2e PDF WebKit + parcours wizard GTK sous xvfb ou DISPLAY) :

```bash
./scripts/test.sh
# ou seulement le smoke UI :
cargo run -p marius-agenda --features gtk -- --root . --gui-self-test
```

```bash
cargo test

# CLI : lister les périodes
cargo run -p marius-agenda -- --root . periods

# Générer un PDF (session graphique ou xvfb-run)
cargo run -p marius-agenda -- --root . generate --period rentree-premiere-vacances --output ./output

# UI (GTK + Libadwaita)
cargo run -p marius-agenda --features gtk -- --root . --gui
```

Paquet `.deb` (après `cargo install cargo-deb`) :

```bash
./scripts/package-deb.sh
# → target/debian/marius-agenda_0.1.0-1_amd64.deb
```

Installation sur une machine Ubuntu/Debian (résout les deps GTK/WebKit) :

```bash
sudo apt install ./target/debian/marius-agenda_*_amd64.deb
marius-agenda --gui
```

Voir aussi `packaging/debian/README.Debian` (copié dans le paquet).

Snap (Snap Store) :

```bash
sudo snap install snapcraft --classic   # une fois
./scripts/package-snap.sh
# → marius-agenda_0.1.0_amd64.snap
sudo snap install --dangerous ./marius-agenda_*.snap
```

Publication : `docs/snap-store.md`.

Assets : `assets/` (vide — illustrations par projet) et `fonts/` (polices embarquées dans le paquet / snap).

## Roadmap

1. ✅ `agenda-core` + config 2026-2027
2. ✅ `agenda-render` (snapshots HTML)
3. ✅ `agenda-pdf` WebKitGTK
4. ✅ App GTK wizard v1 (5 étapes, génération par période)
5. 🚧 Packaging `.deb` / snap — `verify-packaging.sh` + build release sur VM propre
