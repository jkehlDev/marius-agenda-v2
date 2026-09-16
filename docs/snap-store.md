# Publication Snap Store — marius-agenda

Audience : mainteneur (pas doc utilisateur finale).

Approche technique (GTK/WebKit, pièges) : **`docs/snap-packaging-guide.md`**.

## Prérequis

- Compte [snapcraft.io](https://snapcraft.io/) (Ubuntu One).
- `sudo snap install snapcraft --classic`
- Build recommandé : **LXD** (`snap install lxd` — pas besoin de `lxd init` si déjà configuré).
- Base snap : **core24** + extension `gnome` ; WebKitGTK 6 via content snap `webkitgtk-6-gnome-2404` (voir guide packaging).
- Vignette **Snap Store** : `icon: packaging/icons/marius-agenda-256.png` dans `snap/snapcraft.yaml` (distinct du `Icon=` du `.desktop`).
- **Summary / description** : tenir `summary` et `description` dans `snap/snapcraft.yaml` alignés avec le listing snapcraft.io (source de vérité au prochain `snapcraft upload` ; le dashboard peut aussi être édité à la main — éviter la divergence).

## Build

```bash
./scripts/package-snap.sh
# défaut : snapcraft clean (part) puis pack --use-lxd
# si rebuild incrémental foireux : SNAPCRAFT_CLEAN=1 ./scripts/package-snap.sh
# dev rapide sur la machine hôte :
SNAPCRAFT_BUILD_MODE=destructive ./scripts/package-snap.sh
```

Artefact : `marius-agenda_<version>_amd64.snap` à la racine du dépôt.

Test sans store :

```bash
sudo snap install --dangerous ./marius-agenda_*.snap
marius-agenda --gui
```

## Enregistrer le nom (une fois)

```bash
snapcraft login
snapcraft register marius-agenda
```

Si le nom est pris, choisir un autre `name:` dans `snap/snapcraft.yaml` et ré-enregistrer.

## Publier

```bash
snapcraft upload --release=stable ./marius-agenda_*.snap
```

- Première revue manuelle possible (grade `stable`, confinement `strict`).
- Versions suivantes : incrémenter `version` dans `[workspace.package]` du `Cargo.toml` racine (lue via `adopt-info` au build).

## CI (optionnel)

Snapcraft peut builder sur les workers Canonical :

```bash
snapcraft remote-build
```

Configurer le token sur snapcraft.io → Account → Store preferences.

## Plugs

- `home` : données XDG `~/.local/share/marius-agenda`, PDF dans le home.
- `removable-media` : clés USB / volumes montés.
- Extension `gnome` : GTK 4, WebKitGTK, Wayland/X11, portails fichiers.

Pas de plug `network` (PDF hors ligne).
