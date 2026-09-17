# Marius Agenda — état du projet

Synthèse interne (sept. 2026, tag **v0.1.1**). Détails techniques agents : `docs/agent-work.md`. Roadmap publique : `README.md`.

## Légende maturité

| Niveau | Signification |
|--------|----------------|
| **Mature** | Comportement stable, couvert par tests, utilisable au quotidien |
| **Fonctionnel** | Parcours principal OK, écarts mineurs ou peu de tests sur le chemin réel |
| **Partiel** | À valider manuellement ou polish incomplet |
| **Fragile** | Dépend fortement de l’environnement (WebKit, DISPLAY, chemins) |

## Domaine & données (`agenda-core`)

| Feature | Maturité | Notes |
|---------|----------|--------|
| Config JSON (camelCase, année 2026-2027) | **Mature** | `load_config` / `save_config`, défauts calendrier |
| Calendrier, 5 périodes scolaires | **Mature** | Tests + CLI `periods` |
| Vacances, jours cochés, validation wizard | **Mature** | `validate_wizard_step` |
| Imposition livret (duplex odd/even/both) | **Mature** | Core + render |
| Chemins (`MARIUS_AGENDA_DATA`, XDG, install) | **Fonctionnel** | `resolve_data_dir`, `resolve_app_root` |
| Projets `.marius` (ZIP), récents, workspaces | **Fonctionnel** | `agenda-core::project`, GUI accueil |
| Persistance étape wizard | **Fonctionnel** | Fichier `wizard-step` + champ archive |

## Rendu HTML (`agenda-render`)

| Feature | Maturité | Notes |
|---------|----------|--------|
| Templates + CSS impression | **Mature** | Snapshots HTML |
| Images en data-URI | **Fonctionnel** | Pipeline images |
| Livret imposé / page à page | **Fonctionnel** | e2e PDF |

## Images (`agenda-images`)

| Feature | Maturité | Notes |
|---------|----------|--------|
| Import → N&B WebP | **Fonctionnel** | Crate `image`, ton N&B configurable |
| Slots `sourcePath` / `printPath` | **Mature** | Pipeline + `GtkFileDialog` |

## PDF (`agenda-pdf`, `agenda-pipeline`)

| Feature | Maturité | Notes |
|---------|----------|--------|
| HTML → PDF | **Fonctionnel** | WebKitGTK 6 print-to-file ; e2e multi-périodes |
| Runtime PDF | **Mature Linux** | `libwebkitgtk-6.0` (deb) ou content snap WebKit |
| Génération CLI | **Fonctionnel** | `generate --period` ; xvfb si headless |
| Wizard : export sur thread GTK | **Fonctionnel** | `export_prepared_pdf_generation` |

## Application (`marius-agenda`)

| Feature | Maturité | Notes |
|---------|----------|--------|
| CLI `periods` / `generate` | **Mature** | |
| Accueil + wizard 5 étapes (GTK 4 + Libadwaita) | **Fonctionnel** | Self-test GTK, screenshot tour |
| Pills (étapes déjà visitées) | **Fonctionnel** | `wizard_nav::can_goto_step` |
| Illustrations, étape Générer | **Fonctionnel** | Aide duplex numérotée ; `AdwBanner` pendant PDF |
| Dialogues fichiers | **Fonctionnel** | `GtkFileDialog` (images, dossier PDF, `.marius`) |
| Install sans `--root` | **Partiel** | Valider `verify-deb.sh` sur VM propre |

## Packaging

| Canal | Maturité | Notes |
|-------|----------|--------|
| `test.sh` | **Mature** | Unit + e2e PDF + GUI xvfb |
| `.deb` | **Partiel** | `package-deb.sh`, `verify-deb.sh` |
| Snap | **Fonctionnel** | `package-snap.sh`, upload store (revue stable) |

## Risques connus

1. **WebKit / DISPLAY** — PDF et GUI nécessitent un main loop GTK ; CLI headless = xvfb.
2. **UI GTK** — Rebuild conditionnel des panneaux (`refresh_step_page`) ; coût focus clavier possible.
3. **Polish visuel** — Wizard fonctionnel ; typo/espacements perfectibles vs « produit grand public ».
4. **Packaging deb** — Preuve install machine propre encore à boucler (`verify-deb.sh`).

## Pistes polish (sans refonte)

| Priorité | Action |
|----------|--------|
| P0 | VM propre + `dpkg -i` + `--gui` sans `--root` |
| P1 | Vignettes illustrations (`GtkPicture`) si besoin UX |
| P2 | Moins de rebuild wizard ; feedback erreurs WebKit en dialog |
| P3 | Snap stable store ; métadonnées listing à jour |

## Synthèse

Application Rust **GTK + WebKitGTK** avec cœur métier testé, projets `.marius`, packaging deb/snap avancé ; release grand public = surtout **validation install** et **polish UX**.
