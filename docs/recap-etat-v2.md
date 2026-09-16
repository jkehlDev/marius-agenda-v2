# marius-agenda-v2 — Récap features, maturité et phase polish

Document de synthèse (sept. 2026, tag **v0.1.0**). Comparaison de référence : **marius-agenda v1** (Electron + React + Playwright).

Légende maturité :

| Niveau | Signification |
|--------|----------------|
| **Mature** | Comportement aligné v1, couvert par tests automatisés, utilisable au quotidien en dev |
| **Fonctionnel** | Parcours principal OK, écarts mineurs ou peu de tests sur le chemin réel |
| **Partiel** | Squelette ou parité incomplète, à valider manuellement |
| **Fragile** | Dépend fortement de l’environnement (WebKit, zenity, chemins) |
| **Absent** | Non porté ou hors périmètre actuel |

---

## 1. Features reprises et recodées à la main

### Domaine & données (`agenda-core`)

| Feature | v1 | v2 | Maturité | Notes |
|---------|----|----|----------|--------|
| Config JSON (camelCase, année 2026-2027) | ✅ | ✅ | **Mature** | `load_config` / `save_config`, défauts = v1 |
| Calendrier, périodes scolaires (5) | ✅ | ✅ | **Mature** | Tests unitaires + CLI `periods` |
| Vacances, jours cochés, validation wizard | ✅ | ✅ | **Mature** | `validate_wizard_step` calqué sur React |
| Imposition livret (duplex odd/even/both) | ✅ | ✅ | **Mature** | Logique dans core + render |
| Chemins data (`MARIUS_AGENDA_DATA`, XDG) | partiel | ✅ | **Fonctionnel** | `resolve_data_dir` + `resolve_app_root` pour install |
| Persistance étape wizard | sessionStorage | fichier `wizard-step` | **Fonctionnel** | Plus durable que v1 ; pas de sync multi-fenêtre |

### Rendu HTML (`agenda-render`)

| Feature | v1 | v2 | Maturité | Notes |
|---------|----|----|----------|--------|
| Templates + CSS (shared / imposed) | ✅ | ✅ | **Mature** | Snapshots HTML sur 1ère période + ordre lecture |
| Images en data-URI dans le HTML | ✅ | ✅ | **Fonctionnel** | Fidélité bonne ; pas de diff visuel systématique vs v1 |
| Livret imposé | ✅ | ✅ | **Fonctionnel** | e2e PDF sur livret + page à page |

### Images (`agenda-images`)

| Feature | v1 | v2 | Maturité | Notes |
|---------|----|----|----------|--------|
| Import → N&B WebP (cover, activities, weekdays) | Sharp | `image` crate | **Fonctionnel** | Pipeline proche v1 ; pas de benchmark pixel-perfect |
| Slots config `sourcePath` / `printPath` | ✅ | ✅ | **Mature** | Intégré pipeline + GTK FileDialog |

### PDF (`agenda-pdf` + `agenda-pipeline`)

| Feature | v1 | v2 | Maturité | Notes |
|---------|----|----|----------|--------|
| HTML → PDF | Playwright / Chromium | **WebKitGTK 6** print-to-file | **Fonctionnel** | e2e 1 + 5 périodes ; export sur thread GTK principal dans le wizard |
| Dépendance runtime PDF | Chromium embarqué | `libwebkitgtk-6.0` (deb) | **Mature Linux** | Plus de subprocess Chrome ; incident RAM 48 Go supprimé avec l’ancien stack |
| Génération CLI multi-périodes | ✅ | ✅ | **Fonctionnel** | `generate --period` (répétable) |
| Dossier sortie | picker UI | zenity/kdialog + config | **Fragile** | Pas de fallback GTK natif si zenity absent |

### Application (`marius-agenda`)

| Feature | v1 | v2 | Maturité | Notes |
|---------|----|----|----------|--------|
| CLI `periods` / `generate` | N/A (via app) | ✅ | **Mature** | Tests intégration |
| Wizard 5 étapes | React | GTK 4 + Libadwaita | **Fonctionnel** | Parcours complet ; rendu visuel plus « brut » que v1 |
| Pills navigation (étapes déjà visitées) | ✅ | ✅ | **Fonctionnel** | + self-test GTK |
| Erreurs validation inline | liste sous Suivant | label bas de fenêtre | **Fonctionnel** | Pas de dialog au Suivant ; pas de re-validation live à chaque frappe |
| Hero + textes d’aide | CSS soigné | labels GTK | **Partiel** | Contenu repris ; typo/espacements pas au niveau v1 |
| Illustrations (groupes début / jours) | ✅ | ✅ | **Fonctionnel** | Pas de vignette / preview image |
| Étape Générer (mode + périodes) | ✅ | ✅ | **Fonctionnel** | **Liste d’étapes duplex détaillée v1 absente** (ol « imprime impaires… ») |
| Sauvegarde config | à chaque champ | sync + Suivant/Précédent/pill + toggles livret | **Fonctionnel** | Modèle différent mais cohérent |
| GUI sans `--root` installé | implicite | `resolve_app_root` | **Partiel** | À valider après `dpkg -i` sur machine propre |

### Packaging & CI

| Feature | v1 | v2 | Maturité | Notes |
|---------|----|----|----------|--------|
| `test.sh` (unit + e2e + GUI xvfb) | partiel | ✅ | **Mature** | Gate locale = CI GitHub |
| `.deb` cargo-deb | electron-builder | `package-deb.sh` | **Partiel** | Build OK chez dev ; **pas de preuve install VM propre** dans le repo |
| Snap | ✅ | brouillon `snapcraft.yaml` | **Partiel** | Non testé bout-en-bout |
| Dépendance PDF release | Chromium bundle | WebKitGTK via `$auto` deb | **Partiel** | `.deb` plus léger ; dépend du stack GNOME/WebKit distro |

---

## 2. Regard critique sur l’état de l’app

### Ce qui tient bien

- **Séparation des crates** : le domaine est testable sans GTK ; la navigation wizard est partiellement extraite (`wizard_nav`, `agenda-core::wizard`).
- **Fidélité métier** : mêmes règles de validation, mêmes périodes par défaut, même chaîne HTML→PDF.
- **Filet de tests** : plus solide que beaucoup de ports UI (e2e PDF, CLI generate conditionnel, self-test wizard, métadonnées packaging).

### Risques et faiblesses

1. **WebKit / DISPLAY** — Le PDF nécessite WebKitGTK (dépendance `.deb` normale) et un main loop GTK ; CLI headless = xvfb. Valider rendu pixel vs ancien Chromium sur un échantillon de périodes.
2. **UI GTK** — Reconstruction complète de la page à chaque action (`refresh_step_page` vide le `GtkBox` et recrée les widgets). Simple pour les agents et la maintenance, **coûteux** et source de perte de focus clavier ; loin du confort React contrôlé.
3. **Parité visuelle v1** — Pas de feuille de style dédiée type `shared.css` côté app : l’expérience est fonctionnelle, pas « produit fini ».
4. **Dialogs fichiers** — Sortie PDF et dossiers : zenity/kdialog uniquement. Sur Wayland minimal ou serveur sans GUI, l’UI de génération bloque sans message aussi clair que v1.
5. **Données embarquées** — `data/` est gitignoré ; les WebP print par défaut doivent exister ou être régénérés au premier run. Un clone frais sans copie v1 peut surprendre si les chemins config pointent vers des fichiers manquants.
6. **Packaging** — `.deb` sans binaire navigateur vendored ; **installable** si deps GTK/WebKit présentes sur la distro.
7. **Duplex / livret** — Manque l’aide pas-à-pas v1 (liste numérotée selon odd/even/both) ; risque de mauvaise impression pour les utilisateurs non techniques.

### Dette technique assumée (volontaire)

- Code orienté **agents** (peu de commentaires, modules courts, pas d’abstractions « enterprise »).
- Duplication limitée mais **deux chemins sortie** : `resolve_output_dir` (CLI) vs `resolve_output_dir_for_generate` (UI stricte).
- GTK : pas de tests visuels automatisés au-delà du self-test de navigation.

---

## 3. Phase polish / optimisation / raffinement (proposition)

Objectif : passer de **v0.1 utilisable en dev** à **v0.2 installable et confortable**, sans refonte.

### P0 — Consolidation « produit »

| Action | Pourquoi |
|--------|----------|
| Valider `dpkg -i` sur VM/container propre + `marius-agenda periods` / `--gui` sans `--root` | Fermer la boucle packaging (phase E) |
| Documenter dépendance `libwebkitgtk-6.0` dans README utilisateur | Aligner attentes install `.deb` |
| Documenter prérequis UI (zenity) et message GTK si picker indisponible | Réduire les échecs silencieux |

### P1 — Parité UX v1 (faible coût, fort gain)

| Action | Pourquoi |
|--------|----------|
| Reprendre le bloc **duplex-steps** (texte selon `duplex_pass`) dans l’étape Générer | Dernière grosse lacune fonctionnelle vs v1 |
| Vignettes illustrations (optionnel : `GtkPicture` depuis `print_path`) | Confiance utilisateur sur les imports |
| Classe CSS erreur (`error` déjà posée) + espacement barre bas | Lisibilité des validations |

### P2 — Optimisation technique

| Action | Pourquoi |
|--------|----------|
| Réduire les `refresh_step_page` complets : ne reconstruire que l’étape courante ou mettre à jour les champs sans tout détruire | Perf + focus entries |
| Unifier persistance : une fonction `flush_wizard_edits(state)` appelée partout | Moins d’oublis `save_config` |
| Aligner CLI/UI sur **un seul** resolve output (documenter l’écart si voulu) | Moins de bugs « ça marche en CLI pas en GUI » |
| `generate_period` : progression + erreurs WebKit en dialog (pas fenêtre grisée sans feedback) | Polish erreurs |

### P3 — Raffinement optionnel

| Action | Pourquoi |
|--------|----------|
| Snap testé + même layout FHS que le deb | Deux canaux install |
| Comparaison visuelle PDF v1 vs v2 sur 1 période (script diff) | Preuve marketing / confiance |
| i18n : déjà FR ; extraire chaînes wizard si réutilisation | Seulement si multi-langue un jour |

---

## 4. Synthèse une phrase

**v2 est un port Rust crédible du cœur métier et du PDF**, avec un wizard GTK complet mais encore **« ingénieur »** : tests solides, packaging entamé, **polish UX** (progression génération, parité visuelle) avant release grand public.

---

*Maintenir en cohérence avec `docs/agent-work.md` (détails techniques agents) et la roadmap du `README.md`.*
