# Guide empaquetage Snap — apps GTK 4 + WebKitGTK 6

Audience : mainteneurs / agents. Complète `docs/snap-store.md` (publication).

## Principes (synthèse doc Canonical + forum)

| Principe | Pourquoi |
|----------|----------|
| **Une seule pile GTK à l’exécution** | Mélanger GTK du `gnome-platform` et GTK tiré par `stage-packages` (souvent via WebKit) → `undefined symbol` (ex. `gdk_cicp_*`, `gtk_file_dialog_*`). |
| **Extension `gnome` pour GTK/GLib/portails** | Fournit GTK 4 récent, `desktop-launch`, plugs bureau. Ne fournit **pas** WebKitGTK 6 sur core22/core24. |
| **WebKitGTK 6 via content snap dédié** | [`webkitgtk-6-gnome-2404`](https://snapcraft.io/webkitgtk-6-gnome-2404) + SDK au build — modèle [Wike](https://github.com/soumyaDghosh/wike-snap), [webkitgtk-sdk](https://github.com/snapcrafters/webkitgtk-sdk). |
| **Éviter `stage-packages` WebKit/GTK** si `gnome` + content WebKit | Sinon dépendances Debian dans `$SNAP/usr/lib` **écrasent** celles du platform (classique sur le forum). |
| **Part `cleanup` (optionnel)** | Retirer du `prime` les `.so` déjà présents dans `gnome-platform` / content snaps. |
| **`LD_LIBRARY_PATH` : préfixer, pas remplacer** | Utiliser `$SNAP/...${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}` pour garder chemins snapd/GL (doc forum snapcraft). |
| **DBus : slot explicite** | Si `application_id` (ex. `com.jkehldev.mariusagenda`), déclarer un `slots: dbus` session — sinon AppArmor refuse le nom sur le bus. |
| **Plugs WebKit** | `network` + `network-status` souvent requis pour WebKitGTK en snap. |
| **Base `core24` + `platforms:`** | Syntaxe actuelle snapcraft ; alignée Ubuntu 24.04 / GNOME 46. |
| **Build reproductible** | LXD (`snapcraft pack --use-lxd`), version via `craftctl set version` (pas `tomllib` sur Python 3.10). |
| **« Tout embarquer »** | Valable pour apps **sans** extension (pile autonome). Avec GTK bureau, **hybride** (gnome + content WebKit) est la pratique recommandée — tout `stage-packages` GTK+WebKit gonfle le snap (~80 Mo+) et recrée les conflits. |

## Workflow systématique (marius-agenda)

1. **Métadonnées** : `snap/snapcraft.yaml`, `snap/gui/*.desktop`, icône 256px.
2. **Build** : `./scripts/package-snap.sh` → `marius-agenda_<ver>_amd64.snap`.
3. **Vérif layout** : `./scripts/verify-snap.sh`.
4. **Test local** :
   ```bash
   sudo snap install --dangerous ./marius-agenda_*.snap
   snap run marius-agenda --gui
   snap run marius-agenda --gui-self-test   # CI / headless + xvfb
   ```
5. **Debug runtime** :
   ```bash
   snap run --shell marius-agenda
   ldd $SNAP/usr/bin/marius-agenda
   ```
6. **Publication** : `docs/snap-store.md`.

## Pièges rencontrés sur ce projet

| Symptôme | Cause | Fix |
|----------|--------|-----|
| `libwebkitgtk-6.0.so.4` introuvable | GNOME 42 = WebKit 4.x seulement | Content snap WebKit 6, pas `stage-packages` seul sur core22. |
| `gtk_file_dialog_select_folder` | Binaire lié GTK ≥4.10, runtime GTK 4.6 (jammy) | core24 + gnome-46 **ou** ne pas mélanger avec vieux GTK stagé. |
| `gdk_cicp_params_*` + `libmedia-gstreamer.so` | GTK 4.14 dans le snap + modules GTK 4.18 du `gnome-platform` | Retirer GTK/WebKit des `stage-packages` ; WebKit via content snap uniquement. |
| `DBus … AppArmor … com.jkehldev.mariusagenda` | Pas de slot dbus | `slots:` + `name:` = `application_id`. |
| `dirname` / `snapctl` / `date: command not found` + « not connected to gnome-46-2404 » | `PATH` limité à `webkitgtk-platform/usr/bin` | Ne pas surcharger `PATH` au runtime ; WebKit via `LD_LIBRARY_PATH` seulement (cf. Wike). |
| `not connected to gnome-46-2404` (install locale) | Interfaces content pas auto-connectées | `snap connect marius-agenda:gnome-46-2404` (souvent auto après fix PATH). |
| `Icon 'marius-agenda' not found in prime` | `Icon=` = nom thème sans fichier à la racine de `prime/` | `Icon=${SNAP}/meta/gui/marius-agenda.png` + `.desktop`/png dans `meta/gui` au prime. |
| `Could not create GBM EGL display: EGL_NOT_INITIALIZED` | WebKit DMA-BUF / GPU en snap (souvent NVIDIA) | `WEBKIT_DISABLE_DMABUF_RENDERER=1` + `GSK_RENDERER=cairo` (yaml + `prepare_gtk_runtime` si `$SNAP`). |
| Sélecteur de dossier ouvre `~/snap/<app>/…` | `HOME` snap ≠ home utilisateur | `picker_start_dir` → `SNAP_REAL_HOME` (`user_home_for_file_dialog`). |
| `Cargo.toml` introuvable en LXD | `CRAFT_PART_SRC` vide après `clean` partiel | Builder depuis `CRAFT_PROJECT_DIR`. |
| Version snap vide | `tomllib` absent en core22 | `grep`/`sed` sur `Cargo.toml` ou version fixe. |

## Références

- [The WebkitGTK Problem (forum)](https://forum.snapcraft.io/t/the-webkitgtk-problem/35563)
- [webkitgtk-sdk (Snapcrafters)](https://github.com/snapcrafters/webkitgtk-sdk)
- [GNOME extension (doc)](https://ubuntu.com/docs/snapcraft/latest/reference/extensions/gnome-extension/)
- [Manage dependencies](https://documentation.ubuntu.com/snapcraft/latest/how-to/crafting/manage-dependencies/)
- [GTK4 applications (forum)](https://forum.snapcraft.io/t/gtk4-applications/32266)

## Fichiers repo

| Fichier | Rôle |
|---------|------|
| `snap/snapcraft.yaml` | Définition snap ; `icon:` → vignette Snap Store (≠ `Icon=` du `.desktop`) |
| `packaging/icons/marius-agenda-256.png` | Icône store (256×256, &lt; 256 Ko) |
| `scripts/package-snap.sh` | bake icônes + `snapcraft pack` |
| `scripts/verify-snap.sh` | binaire + assets dans le `.snap` |
| `docs/snap-store.md` | login / register / upload |
