# Agent work log — marius-agenda-v2

Audience: **AI agents** maintaining this repo. Not user-facing docs.

## Product target

- School agenda generator: JSON config, HTML/CSS print layout, 5-step wizard, `.marius` projects.
- **GTK 4 + Libadwaita** on Linux (no embedded Chromium).
- PDF: **WebKitGTK 6** only (`agenda-pdf` → `webkit.rs`).
- Default config: school year **2026-2027** only (no legacy migration).

## Layout (grep from crate name)

| Path | Role |
|------|------|
| `crates/agenda-core` | Config, calendar, wizard validation, paths, store, `resolve_output_dir_for_generate` |
| `crates/agenda-render` | HTML/CSS render + snapshot tests |
| `crates/agenda-images` | B&W WebP pipeline, `import_illustration_file` |
| `crates/agenda-pdf` | `export_pdfs` via WebKitGTK print-to-PDF |
| `crates/agenda-pipeline` | `prepare_pdf_generation`, `export_prepared_pdf_generation`, e2e PDF test |
| `apps/marius-agenda` | CLI + GTK (`feature gtk`), `wizard_nav` pure logic |
| `scripts/test.sh` | **Gate**: workspace tests (xvfb for WebKit PDF) + GTK `--gui-self-test` |

## Path resolution (do not duplicate)

- **App root**: `agenda_core::resolve_app_root(cli_root)` — dev tree with `assets/`+`fonts/`, else `/usr/share/marius-agenda` or exe-relative share.
- **User data**: `resolve_data_dir(root)` — `MARIUS_AGENDA_DATA` → `{root}/data` if bundled assets → else XDG `~/.local/share/marius-agenda/data`.
- **Wizard step file**: `{data_dir}/wizard-step` — API `load_wizard_step` / `save_wizard_step` (tests hors projet).
- **Projets `.marius`**: `manifest.json` → `wizard_step` (défaut 0 si absent) ; sauvé à l’enregistrement ; restauré à l’ouverture (clamp 0..4).

## Projects (GUI) — `.marius` only (implemented)

- **Format**: archive ZIP `*.marius` (`agenda-core::project`) — pas de projet-dossier, pas de cloud.
- **Workspace**: cache XDG (`workspaces_cache_dir`) pendant l’édition ; `save_project_archive` / `open_project_archive`.
- **Home**: nouveau / ouvrir `.marius` / récents ; wizard après ouverture.
- **Récents**: `load_recent_projects()` retire les `.marius` absents **et** les archives de test sous `$TMP/marius-it-*` / `marius-proj-*` (suppression du dossier scratch + JSON) ; rafraîchi à l’accueil via `home::refresh_recent_list` ; poubelle → `confirm_destructive` + `delete_project_archive` (fichier + entrée JSON) ; tests avec `MARIUS_AGENDA_SKIP_RECENT=1`.
- **Cache workspaces**: `~/.cache/marius-agenda/workspaces/ws-{pid}-*` — supprimé à la fermeture / retour accueil / changement de projet ; au lancement GUI, `prune_stale_workspace_dirs()` enlève les dossiers des anciens PID.
- **Header**: accueil + menu Enregistrer (`win.save` / `win.save-as`) **désactivés** sur l’écran d’accueil (`session::sync_header_chrome`) ; **Ctrl+S** = enregistrer (wizard uniquement).
- **Screenshot tour**: `--gui-screenshot-tour` → `step-00-home.png` … `step-05` ; sortie via `scripts/ui-screenshot-tour.sh` (défaut `/tmp/marius-ui-screenshots`, non versionné).
- **Décisions produit**:
  - Fermeture sans enregistrer → **perdre les modifs** (dialogue Enregistrer / Ne pas enregistrer / Annuler).
  - PDF sans projet enregistré sur disque → **autorisé** (workspace cache suffit pour `data_dir`).
  - Nouveau projet → `create_blank_project_config()` (= calendrier 2026-2027, illustrations vides ; `assets/` repo vide, pas d’IP tierce).
- **dirty**: non persisté dans le `.marius` ; titre fenêtre + `*` ; pas d’auto-save global vers l’archive.

## PDF + GTK threading

- HTML render + images: `prepare_pdf_generation` (may run off UI thread).
- WebKit print: **GTK main thread** — `export_prepared_pdf_generation` after prepare (`ui/generate.rs`, `idle_add_local` + mpsc).
- CLI `generate`: needs DISPLAY or xvfb.

## GTK wizard architecture

- **State**: `ui/state.rs` — `AppState` + `apply_step_sync`.
- **Navigation**: `ui/navigation.rs` — `perform_wizard_*`; `refresh_step_page(ui, state, force_rebuild)`.
- **Panels**: `ui/steps.rs` — body built on first visit or `force_rebuild` (not on every pill/prev/next).
- **Layout/CSS**: `ui/layout.rs`, `apps/marius-agenda/ui/marius.css` — clamp 880px, cartes périodes, previews `GtkPicture`.
- **File dialogs**: `ui/util.rs` — `GtkFileDialog` for images and output folder (**no zenity**).
- **Generate**: `ui/generate.rs` — `output_dir` dans `config.json` ; dossier absent → dialogue « Choisir… » / Annuler au clic période ; `set_project_output_dir` (+ auto-save `.marius` si déjà enregistré) ; `AdwBanner` pendant PDF.
- **Borrow rule**: never `borrow_mut` across `refresh_step_page`; use `state::edit_state_then` for mutate-then-refresh; clone `WizardUi` for callbacks.
- **Tests**: `agenda-core::project` (récent, workspaces, skip env) ; `ui/state.rs` unit (`edit_state_then`, `apply_step_sync`) ; GTK `--gui-self-test` inclut refresh étape Générer ; `apps/marius-agenda/tests/project_recent.rs`.

## Phase G — UI refactor (in progress)

| Item | Status | Notes |
|------|--------|-------|
| GtkFileDialog dossier PDF | done | removed `native_dialog` / zenity |
| Refresh step body conditionnel | done | `last_filled_step` + `force_rebuild` |
| Bannière génération PDF | done | `AdwBanner` |
| `.desktop` dans deb | done | `marius-agenda.desktop` |
| Duplex help (liste numérotée) | done | step 5 |
| GtkFileDialog déjà pour images | done | pre-existing |
| Polish graphique wizard (clamp, grilles, previews) | done | `ui/layout.rs`, `ui/marius.css`, `--gui-screenshot-tour` |
| HIG lot A (menu, About, shortcuts, StatusPage, pills CSS) | done | `ui/chrome.rs`, `ui/icons.rs` |
| Packaging VM propre `dpkg -i` | in progress | `package-deb.sh`, `verify-deb.sh`, deps WebKit explicites ; `.desktop` `%f` + MIME `.marius` |
| Snap Store | in progress | `core24` + `gnome` + `webkitgtk-6-gnome-2404` ; WebKit PDF : `WEBKIT_DISABLE_DMABUF_RENDERER` + `GSK_RENDERER=cairo` ; `summary`/`description` = listing store |
| README utilisateur dans paquet | done | `packaging/debian/README.Debian` |

## Progress phases (legacy)

| Phase | Status | Notes |
|-------|--------|-------|
| A–D Core + wizard | done | |
| E Packaging | in progress | `.desktop`, `$auto` deps |
| F Release polish | pending | user README |

## Decisions (do not revert without reason)

- Wizard validation in `agenda-core::wizard` (single source).
- PDF backend WebKitGTK only.
- Folder picker is GTK-only when GUI is built (no zenity fallback).

## Commands

```bash
./scripts/test.sh
cargo run -p marius-agenda --features gtk -- --root . --gui
```
