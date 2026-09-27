# Ditto — validation du 28 septembre 2026

## Sélecteur de couleurs et couverture de régression — 28 septembre 2026

Branche `feat/color-picker-cursor-tests`, base `631a1ae` (petits L braille inclus).
Version publique 0.1.0 / X0005 et format projet V6 conservés ; aucune nouvelle
release créée.

- Cause reproduite : l’outil Guides/Gomme guides mémorisé supprimait le cadre de
  cellule même en mode clavier. Deux tests de géométrie émise échouaient avant
  la correction, dont l’aller-retour par la couleur des guides.
- Deuxième correction : valider une modale après Tab restaure maintenant le focus
  du canevas. Le passage clavier recadre aussi un curseur sorti de la vue par pan.
- Sélecteur HSV commun : FG, BG, palette, guide, thème et encres Duotone ; hex
  synchronisé, avant/après, glissement borné, clavier et contrôle accessible de teinte.
- `scripts/check.sh --native` : format, Clippy strict et **128 tests passent**
  (les deux tests GPU explicitement exécutés), puis deux parcours natifs réussis.
  31 tests supplémentaires, dont plusieurs matrices de scénarios et balayages
  de couleurs, formes, masques et historique mixte.
- Apple M2 / Metal / SF Mono : captures du picker et du curseur inspectées ;
  11 captures supplémentaires vérifiées par sondes GPU, en plus des scènes
  guides/shaders/exports existantes. Réouverture et export sans aides vérifiés.
- Preuves locales : `validation/runtime/color-picker/check-native.log`,
  `validation/runtime/color-picker/native/` et
  `validation/runtime/native/run-smw4rp_s/`. Bundle de travail : `dist/Ditto-dev.app`.
- CI étendue avec Xvfb/Mesa sous Ubuntu : même commande de validation et artefacts
  conservés même en cas d’échec. Les détails et limites sont dans [docs/TESTS.md](docs/TESTS.md).
- Les entrées sont injectées dans le routage applicatif. Ce résultat ne prétend
  pas valider une frappe système physique, les dialogues OS, le presse-papiers
  inter-applications ou une session complète VoiceOver/AT-SPI/Wayland.

## Petits L braille de hauteur 3 — 28 septembre 2026

- Modification réalisée dans le worktree `ditto-braille-l`, branche
  `feat/braille-l-corners`, depuis `dbd08705112940ce0b8423b0cf7344aca9ad0118`.
- Banque « 3 points » : huit coins en L, quatre orientations alignées en haut
  et en bas. C/V/B/N → ⡖/⢲/⣆/⣰ ; 0/1/2/3 → ⠏/⠹/⠧/⠼.
  30 formes dans cette banque, 156 dans le preset, aucun ancien mapping déplacé.
- `cargo test --locked` : 95 tests passent, deux tests GPU ignorés par défaut.
  Le test du catalogue vérifie les huit géométries exactes et leurs touches ;
  les vérifications globales de symétries, unicité et accès aux glyphes passent.
- `cargo fmt --all --check`, clippy strict et `git diff --check` : OK.
- Planche PNG inspectée et roundtrip projet vérifié :
  `validation/runtime/braille-l-corners/braille.png` dans le worktree.
- Bundle macOS release signé ad hoc ; smoke exécuté sur ce bundle,
  Apple M2 / Metal / SF Mono, édition/export et composition des guides : OK.
  Journal : `validation/runtime/braille-l-corners/native.log`.
  Cette recette ne constitue pas une frappe physique ; Linux non exécuté.
- Version publique et BUILD_ID conservés ; ajout documenté dans « À paraître »
  du changelog, sans publier de release GitHub.


## Release publique 0.1.0 — X0005 — 28 septembre 2026

- Dépôt public créé : https://github.com/Mars7attack/ditto. Le premier commit
  `04da4e2985e86a922be840931d6e2ca16c375695` a été poussé **avant** les modifications
  de position et recoloration des guides. Les binaires, brouillons et captures
  locales restent hors Git. Licence MIT, changelog, BUILD_ID et CI sont suivis.
- Version publique 0.1.0 et identifiant interne X0005 : contrôlés via `--version`,
  affichés dans l’aide et incorporés aux paquets. Les anciennes versions 0.2–0.5
  sont des étapes locales, indépendantes de la numérotation publique.
- **97 tests passent**, dont les deux tests GPU explicites. Guide par défaut
  derrière les caractères, migration des guides V5, sauvegarde V6, couleur de
  tous les traits, conservation des points/épaisseurs/cellules, undo/redo et absence
  de révision lors d’une recoloration identique sont vérifiés.
- Formatage et clippy local strict : OK. La première CI GitHub a détecté deux
  nouveaux lints de Rust 1.98 sur les pixels d’atlas ; les itérations ont été
  adaptées sans changer le rendu. L’issue courante du workflow macOS/Linux est
  consultable dans https://github.com/Mars7attack/ditto/actions/workflows/ci.yml.
- Smoke natif du bundle release réussi sur Apple M2 / Metal / SF Mono.
  Contrôle pixel par pixel du framebuffer sur cinq captures : référence bleue,
  guides verts devant la référence ; caractère rouge devant le guide par défaut ;
  inversion via l’option ; répétition avec shaders ; recoloration du guide en jaune.
  Les couleurs mesurées correspondent exactement aux valeurs attendues.
- L’export avec guides/référence reste pixel-identique à celui du dessin seul.
  Projet roundtrip V6 et état du calque vérifiés. Le panneau des guides est inspecté
  visuellement et ses contrôles sont testés à la taille minimale de fenêtre.
- Paquet macOS signé ad hoc vérifié, sans notarisation. Linux x86_64/glibc 2.35
  cross-compilé avec la toolchain isolée ; pas de session graphique Linux.
- Preuves locales sous `validation/runtime/X0005/` : `tests.log`, `clippy.log`,
  `macos-build.log`, `linux-build.log`, `release.log` et, sous `release/`, les
  captures `guide-*.png`, le rapport `guide-compositing.txt` et `guide-order.ditto`.
  Le smoke utilise les routes internes de la fenêtre et le framebuffer réel ;
  aucune nouvelle recette de souris physique ni VoiceOver n’est revendiquée.



## Guides, recoloration et réglages 0.5.0 — 27 septembre 2026

- **95 tests passent**, y compris les deux tests GPU exécutés explicitement avec
  `cargo test --locked -- --include-ignored`. Journal :
  `validation/runtime/workspace-v050/tests.log`.
- `cargo fmt --check` et clippy strict tous targets : OK.
  Journal : `validation/runtime/workspace-v050/clippy.log`.
- Guides : points sous-cellulaires, interruption en sortie de canevas, annulation
  par geste, perte de focus, calque masqué, gomme balayée, limites, partage des
  cellules par Arc, historique mixte, sauvegarde V5 et rejet des données invalides.
- Recoloration : trajet interpolé, taille, sélection, conservation exacte des
  glyphes/fonds, espaces ignorés, masques du crayon indépendants, sortie/rentrée
  sans segment parasite, undo/redo et annulation du geste.
- Préférences : aperçu isolé du document, sélecteur hexadécimal, annuler/enregistrer,
  relecture au lancement, thèmes lisibles et échec d’écriture sans perte du brouillon.
  Contrôles vérifiés sans chevauchement à 1120 × 720. Raccourcis testés en modes
  souris et clavier ; les touches C/D restent des glyphes en mode clavier.
- Smoke exécuté sur **le bundle release final**, Apple M2 / Metal / SF Mono.
  Il conserve la recette shaders et ajoute guides/recoloration/préférences.
  Journal : `validation/runtime/workspace-v050/release.log`.
- Captures réelles du framebuffer inspectées dans
  `validation/runtime/workspace-v050/release/` : `workspace-guides.png`,
  `workspace-paper.png`, ainsi que panneaux Guides et Réglages en build debug.
  Les exports sont comparés sans/avec guides et restent pixel-identiques.
  Projet de démonstration : `workspace.ditto` ; export : `workspace-export.png`.
- Paquet macOS **0.5.0**, signature ad hoc vérifiée avec `codesign --verify --deep
  --strict`. Archive macOS reconstruite. Cross-build Linux x86_64/glibc 2.35 réussi
  avec la toolchain isolée existante ; journal `workspace-v050/linux-build.log`.
  Archive Linux et sommes SHA-256 actualisées.
- La fenêtre utilisateur existante reste ouverte. Les scénarios utilisent leurs
  propres fichiers de récupération et de préférences. L’attachement CUA à Ditto
  échoue avec `cgWindowNotFound` : aucune recette de clic/drag externe ni VoiceOver
  n’est revendiquée. Le smoke utilise le routage interne des gestes avec une vraie
  fenêtre et un vrai GPU. La GUI Linux n’a pas été exécutée.

Le format de sauvegarde est désormais V5 ; les projets V1 à V4 restent lisibles.
Les couleurs de thème sont des préférences locales et ne modifient pas les exports.

## Motif 367 et homologues 0.4.12 — 27 septembre 2026

- Numérotation par rangées : 1/2, 3/4, 5/6, 7/8.
- Espacés / lignes : 4 → ⡢ (367), 5 → ⢔ (458), 6 → ⠕ (145), 7 → ⠪ (236).
- 34 formes dans cette page, 148 au total, six pages ; anciens mappings conservés.
- 81 tests passent : géométrie exacte, nombre de points, unicité, mappings et
  interface compacte. Deux tests GPU ignorés par défaut ; formatage/clippy : OK.
- Planche exportée inspectée et roundtrip projet vérifié :
  `validation/runtime/braille-367/braille.png`.
- Smoke natif Apple M2 / Metal / SF Mono réussi, capture inspectée :
  `validation/runtime/braille-367/native/braille-window.png`.
  Il vérifie le routage interne, pas une frappe physique.
- Build macOS 0.4.12 signé ad hoc ; cross-build Linux 0.4.12 réussi,
  journal `validation/runtime/linux-build-v0412.log`. GUI Linux non exécutée.

## Motif 3567 et homologues 0.4.11 — 27 septembre 2026

- Numérotation par rangées : 1/2, 3/4, 5/6, 7/8.
- Espacés / lignes : 0 → ⡦ (3567), 1 → ⢴ (4568), 2 → ⠗ (1345), 3 → ⠺ (2456).
- 30 formes dans cette page, 144 au total, six pages ; anciens mappings conservés.
- 81 tests passent, dont géométrie exacte, unicité et accès aux touches ;
  deux tests GPU restent ignorés par défaut. Formatage et clippy strict : OK.
- Export et roundtrip projet vérifiés, planche inspectée :
  `validation/runtime/braille-3567/braille.png`.
- Smoke natif macOS réussi sur Apple M2 / Metal / SF Mono ; interface inspectée :
  `validation/runtime/braille-3567/native/braille-window.png`.
  Ce scénario vérifie le routage interne, pas une frappe physique.
- Build macOS 0.4.11 signé ad hoc ; cross-build Linux réussi,
  journal `validation/runtime/linux-build-v0411.log`. GUI Linux non exécutée.

## Motif 3678 dans Espacés / lignes 0.4.10 — 27 septembre 2026

- L → ⣢ ajouté selon la grille numérotée ligne par ligne : seuls les points
  3, 6, 7 et 8 sont présents. Les symétries sont sur M/W/X et les variantes
  alignées en haut sur C/V/B/N. Les 18 mappings précédents sont conservés.
- Espacés / lignes contient 26 formes ; le charset totalise 140 motifs sur six pages.
- **81 tests standards réussis**, deux tests GPU ignorés par défaut. Le contrôle
  décode le motif de L en coordonnées ligne par ligne et vérifie exactement
  [3, 6, 7, 8], les quatre points de chaque variante, les symétries et l’unicité.
- Formatage, clippy strict, build macOS 0.4.10 et signature ad hoc : OK.
- Planche inspectée : `validation/runtime/braille-3678/braille.png`. Le projet
  de preuve a été exporté puis rechargé sans différence.
- Scénario natif du bundle Apple M2 / Metal réussi ; capture de la page vérifiée :
  `validation/runtime/braille-3678/native/braille-window.png`.
  Journal : `validation/runtime/braille-3678/native.log`.
- Build Linux 0.4.10 réussi ; journal `validation/runtime/linux-build-v0410.log`.
  Archives et SHA-256 reconstruits. Pas de session graphique Linux exécutée.

La fenêtre utilisateur est laissée ouverte. Les scénarios natifs utilisent leurs
propres documents et leurs entrées internes, sans revendiquer une frappe physique.


## Coin évidé 0.4.9 — 27 septembre 2026

L’utilisateur a précisé le point à retirer : bas gauche du motif de la touche 4.

- Espacés / blocs ajoute 6 → ⢫, puis ses symétries 7 → ⡝, 8 → ⣜, 9 → ⣣.
  La page compte 36 formes ; le charset totalise 132 motifs sur les mêmes six pages.
- **81 tests standards réussis**, deux tests GPU ignorés par défaut. Le contrôle
  compare les masques des touches 4 et 6 : seul le bit du point bas gauche change.
  Il vérifie aussi les cinq points restants, les symétries, l’unicité et les mappings.
- Formatage, clippy strict, build macOS 0.4.9 et signature ad hoc : OK.
- Planche inspectée : `validation/runtime/braille-notch/braille.png`. Le projet
  de preuve a été exporté puis rechargé sans différence.
- Scénario natif du bundle Apple M2 / Metal réussi ; les 36 mappings sont visibles
  dans `validation/runtime/braille-notch/native/braille-spaced-blocks-window.png`.
  Journal : `validation/runtime/braille-notch/native.log`.
- Build Linux 0.4.9 réussi ; journal `validation/runtime/linux-build-v049.log`.
  Archives et SHA-256 reconstruits. Pas de session graphique Linux exécutée.

La fenêtre utilisateur n’a pas été fermée. Les scénarios natifs utilisent leurs
propres documents et leurs entrées internes, sans prétendre à une frappe physique.


## Colonnes 4+3 et coins opposés 0.4.8 — 27 septembre 2026

- Quatre formes à sept points ajoutées à la page 4 points, sur H/J/K/L :
  ⣷/⣾/⡿/⢿. Une colonne est pleine, la seconde a trois points contigus.
- Le motif de coins opposés retenu est ⣫, avec son miroir ⣝ : deux angles de
  trois points et deux trous diagonaux au centre. Ils sont ajoutés sur 4/5 dans
  Espacés / blocs, sans modifier les trente mappings déjà présents.
- Le charset contient 128 cibles sur six pages : 12/25/22/19/18/32.
- **81 tests standards réussis**, deux tests GPU ignorés par défaut. Les contrôles
  existants vérifient les quatre masques à sept points, les deux demi-groupes de
  trois points des coins, leurs symétries, les mappings, l’unicité et l’accessibilité.
  L’exception aux rangées entièrement vides est limitée aux deux nouveaux coins.
- Formatage, clippy strict, build macOS 0.4.8 et signature ad hoc : OK.
- Planche inspectée : `validation/runtime/braille-corners/braille.png`.
  Projet de preuve exporté puis rechargé avec succès. Scénario natif Apple M2 / Metal
  réussi et capture du panneau inspectée :
  `validation/runtime/braille-corners/native/braille-spaced-blocks-window.png`.
  Journal : `validation/runtime/braille-corners/native.log`.
- Build Linux 0.4.8 réussi ; journal `validation/runtime/linux-build-v048.log`.
  Archives et SHA-256 reconstruits. Pas de recette graphique Linux.

Le catalogue Unicode, les identifiants enregistrés et le format de projet ne changent
pas. La fenêtre utilisateur est laissée ouverte ; les scénarios natifs emploient leurs
propres documents et leurs entrées internes.


## Pages braille Espacés 0.4.7 — 27 septembre 2026

Organisation confirmée par l’utilisateur : deux pages voisines, avec les 48 motifs.

- Espacés / lignes regroupe les 18 motifs à extrémités séparées des anciennes
  banques 3 et 4 points. Ces banques retrouvent 22 et 15 motifs continus.
- Espacés / blocs ajoute 30 variantes : un groupe de trois ou quatre points,
  une rangée vide, puis une ligne de deux points ou un point gauche/droit. Toutes
  les orientations du groupe et les inverses haut/bas sont inclus.
- Le charset contient 122 motifs sur six pages (12/25/22/15/18/30). Chaque cible
  reste accessible par une lettre ou un chiffre, sans dépassement des 36 sources.
- **81 tests standards réussis**, deux tests GPU ignorés par défaut. Les contrôles
  couvrent l’absence de trous dans les pages par hauteur, la totalité des motifs
  espacés, leurs symétries, leur unicité et leur accessibilité.
- Formatage, clippy strict, build macOS 0.4.7 et signature ad hoc : OK.
- Planche des six pages inspectée : `validation/runtime/braille-spaced/braille.png`.
  Le projet de preuve a été exporté et rechargé sans différence.
- Scénario natif du bundle Apple M2 / Metal : OK, avec bouclage vers la sixième
  banque, capture des deux pages Espacés et contrôle visuel de la page blocs :
  `validation/runtime/braille-spaced/native/braille-spaced-blocks-window.png`.
  Journal : `validation/runtime/braille-spaced/native.log`.
- Build Linux 0.4.7 réussi ; journal `validation/runtime/linux-build-v047.log`.
  Archives et SHA-256 reconstruits. Pas de recette graphique Linux.

Les mappings des motifs déplacés changent avec leur nouvelle page. Les dessins
et les identifiants de glyphes restent inchangés. La fenêtre utilisateur n’a pas
été fermée ; le scénario natif utilise son propre document et ses entrées internes.


## Compléments braille 0.4.6 — 27 septembre 2026

- Banque 2 points : angles bas K/L/M/W et variantes hautes ajoutés après les
  17 anciens mappings. A/Z/E restent les deux colonnes et le rectangle bas.
- Banques 3 et 4 points : neuf combinaisons d’extrémités séparées par une ou deux
  rangées vides, avec toutes les variantes gauche/droite de 2+2, 1+1, 1+2 et 2+1.
  Les motifs de hauteur 3 sont alignés en bas de la cellule.
- Les banques contiennent 12/25/31/24 motifs, soit 92 au total, toujours dans les
  36 touches directes. Les nouvelles cibles occupent uniquement des touches libres.
- **80 tests standards réussis**, deux tests GPU ignorés par défaut. Le contrôle
  existant des charsets vérifie l’étendue verticale, les symétries à l’intérieur
  de cette étendue, les neuf combinaisons espacées, les angles bas, l’unicité et
  l’accès à chaque cible. Le catalogue général reste à 665 glyphes.
- Formatage, clippy strict, build macOS 0.4.6 et signature ad hoc : OK.
- Planche des quatre hauteurs inspectée :
  `validation/runtime/braille-completion/braille.png` ; export et réouverture du
  projet de preuve réussis. Scénario natif du bundle Apple M2 / Metal réussi et
  capture de la banque 3 points inspectée :
  `validation/runtime/braille-completion/native/braille-window.png`.
- Build Linux 0.4.6 réussi, archives reconstruites et SHA-256 vérifiés. Journal
  `validation/runtime/linux-build-v046.log`. Pas de session graphique Linux exécutée.

Le scénario natif utilise ses documents et entrées internes. La fenêtre utilisateur
reste ouverte et doit être relancée pour charger le nouveau catalogue.


## Blur et Blur des contours 0.4.5 — 27 septembre 2026

- Deux effets ajoutés au catalogue : Blur (rayon, horizontal, vertical) et Blur
  des contours (rayon, seuil, douceur), avec mélange, ordre, annulation et sauvegarde.
- Flou gaussien en alpha prémultiplié. Le masque des contours utilise le contraste
  du voisinage lissé, en couleur et en alpha, afin de conserver les intérieurs de
  faible contraste sans produire des anneaux autour de l’antialiasing.
- **80 tests standards réussis**, deux tests GPU ignorés par défaut ; ces deux
  tests ont été exécutés explicitement et passent aussi (**82 tests distincts**).
  Ils couvrent les onze effets, axes indépendants, diffusion sans bord noir,
  conservation de la somme d’alpha à 1 % près sur la fixture, rayons nuls, mélange
  nul, contours sur fond opaque, détails internes conservés, échelles ×1/×2/×4,
  exports d’une pile mixte et sauvegarde/chargement de projets et de looks.
- Contrôles UI vérifiés à 1120×720 et 1184×832, avec les deux nouveaux effets dans
  une pile pleine. Le catalogue occupe quatre lignes sans collision avec le pied
  de fenêtre ; le nom complet Blur des contours reste lisible dans la pile.
- Formatage, clippy strict, build macOS 0.4.5 et signature ad hoc : OK.
- Scénario natif du bundle sous Apple M2 / Metal : ajout des deux effets, réglages,
  sauvegarde/rechargement V4, export PNG, zoom et déplacement d’aperçu, annulation : OK.
  Capture inspectée : `validation/runtime/blur/native/shaders-zoomed-window.png`.
  Journal : `validation/runtime/blur/native.log`.
- Comparaison Original / Blur / Blur des contours inspectée :
  `validation/runtime/blur/comparison.png`. L’exemple `blur_proof` produit aussi
  trois PNG ×4 et trois projets réouverts et comparés à leurs documents sources.
- Build Linux 0.4.5 réussi ; journal `validation/runtime/linux-build-v045.log`.
  Archives reconstruites et sommes SHA-256 vérifiées. Pas de recette graphique Linux.

Le format reste V4, mais les nouveaux types d’effets nécessitent 0.4.5 pour être
lus. La validation native pilote les entrées internes ; aucune frappe/souris physique
n’est revendiquée. L’instance utilisateur n’a pas été fermée.


## Aperçu shaders navigable 0.4.4 — 27 septembre 2026

- Zoom progressif à la molette/au trackpad dans la zone d’aperçu, ancré sous le
  pointeur ; glisser avec le bouton gauche pour déplacer l’image. Barre −/+, taux,
  Ajuster et 100 %, avec raccourcis +/−, 0, 1 et Alt + flèches.
- État de vue indépendant du canevas et du document, conservé pendant les réglages,
  comparaisons, annulations et réouvertures. Le zoom réutilise les textures déjà
  calculées. Rendu et interactions partagent la même géométrie de découpe.
- `cargo test --locked --quiet` : **79 tests réussis**, un test GPU ignoré par défaut.
  Les régressions couvrent l’ancrage, le scroll limité à l’aperçu, les raccourcis,
  le cadrage conservé, l’absence de peinture pendant le glisser, sa terminaison sur
  relâchement/perte de focus/fermeture, l’isolement du canevas et la fenêtre compacte.
- Formatage, clippy strict, build macOS 0.4.4 et signature ad hoc : OK.
- Le binaire signé a passé le scénario natif Apple M2 / Metal : vue ajustée, zoom
  122,1 %, déplacement, comparaison, réglage puis annulation, avec vérification
  que le document et le cadrage du canevas restent identiques. Capture du panneau
  agrandi inspectée visuellement :
  `validation/runtime/shader-preview-navigation/native/shaders-zoomed-window.png`.
  La vue ajustée est dans `shaders-window.png` ; journal `native.log` dans le dossier parent.
- Build Linux 0.4.4 et archive reconstruits ; journal
  `validation/runtime/linux-build-v044.log`. Pas de session graphique Linux exécutée.

Le scénario natif pilote les méthodes d’entrée et capture le framebuffer présenté ;
la gestuelle sur le trackpad physique n’est pas une recette manuelle effectuée ici.
L’instance de l’utilisateur reste ouverte ; la nouvelle version se charge à la relance.


## Netteté shaders 0.4.3 — 27 septembre 2026

Cause confirmée : l’aperçu shader rasterisait à ×1 (8×16 par cellule), puis
agrandissait cette image alors que le rendu direct utilisait l’atlas SF Mono ×4.
Le passage par cette image réduite perdait les détails avant toute passe d’effet.

- Source d’aperçu à ×4 (32×64 par cellule) quand les budgets GPU le permettent,
  cache cohérent et propagation du facteur aux effets. Replis ×2/×1 pour les très
  grandes grilles ; limites détaillées dans `docs/SHADERS.md`.
- Filtrage final en alpha prémultiplié : les texels transparents ne noircissent
  plus les contours. Le glow échantillonne tous les texels de son rayon, sans les
  trous périodiques produits par le précédent noyau espacé à haute résolution.
- **76 tests standards réussis**, un test GPU ignoré par défaut ; le test GPU a
  été exécuté explicitement et passe aussi (**77 tests distincts validés**).
  Il couvre les neuf effets, l’alpha, l’ordre, les exports et la continuité d’un
  halo autour d’un trait fin à ×4. Le nouveau test de source compare les couvertures
  des lettres/accents/braille à celles de l’atlas, et vérifie les budgets de texture.
- Formatage, clippy strict, build macOS 0.4.3 et signature ad hoc : OK.
- Le binaire signé a passé le scénario natif complet et le scénario de netteté
  sous Apple M2 / Metal, SF Mono, Retina ×2, cellules de 64 points (128 pixels écran).
  Comparaison rendu direct / shader neutre : écart maximal de **1/255 à l’intérieur
  des glyphes**. Les captures directe, neutre et Néon ont été contrôlées.
- Les frontières de cellules sont exclues de cette assertion : la texture aplatie
  interpole entre voisins, tandis que le rendu direct découpe chaque quad. Le rapport
  conserve leurs écarts (maximum 95/255 sur une mince frontière de bloc ; moyenne
  de 0,159/255 sur toute la fixture). Ne pas présenter les deux images complètes
  comme identiques pixel par pixel.
- Le scénario natif complet a duré 0,43 s sur cet hôte, avec environ 244 Mio de RSS
  maximale. Cette mesure courte n’est pas un profilage de frappe sur grille pleine.
- L’export sans effets du scénario correspond au projet rechargé (640×800).
- Build Linux 0.4.3 réussi et archive reconstruite ; session graphique Linux non testée.

Preuves : `validation/runtime/shader-sharpness/release/quality.txt`,
`quality-clean.png`, `quality-neutral.png`, `quality-glow.png`, `quality.ditto`,
`quality.png`, ainsi que `validation/runtime/shader-sharpness/native.log` et
`validation/runtime/linux-build-v043.log`. Les scénarios utilisent leurs propres
documents et brouillons ; l’instance de l’utilisateur n’a pas été fermée.


## Navigation et zoom 0.4.2 — 27 septembre 2026

- Cmd + ↑ / ↓ sur MacBook (Ctrl sous Linux) change la banque précédente/suivante,
  avec bouclage, sans déplacer le curseur ni modifier le document. Le panneau et
  l’aide affichent le raccourci ; Page précédente/suivante reste disponible.
- Zoom progressif selon la distance de scroll : environ 5 % par ligne de molette,
  10,5 % pour 100 pixels logiques de trackpad. Les dimensions et l’origine restent
  fractionnaires pour éviter les sauts et conserver le point sous le pointeur.
- `cargo test --locked --quiet` : **75 réussis**, 1 GPU ignoré par défaut.
  Les nouveaux cas couvrent toutes les banques et leur bouclage, flèches simples,
  dialogues/IME/mode souris, équivalence de 1 et 200 événements de trackpad,
  Retina ×2, ancrage, inversion du geste, limites de zoom, scroll horizontal et
  absence de modification du dessin.
- Formatage et clippy strict : OK. Bundle macOS 0.4.2 signé ad hoc, signature vérifiée.
- Scénario natif du bundle sous Apple M2 / Metal : OK, y compris le raccourci de
  banque et le pas de molette. Capture inspectée :
  `validation/runtime/navigation-zoom/native/braille-window.png`.
  Journal : `validation/runtime/navigation-zoom/native.log`.
  Export PNG identique au projet rechargé (640×800).
- Build Linux 0.4.2 et archives reconstruits ; journal
  `validation/runtime/linux-build-v042.log`. Pas de recette graphique Linux.

La validation des entrées passe par le routage interne du scénario natif ; le ressenti
sur le trackpad physique et les frappes réelles restent à confirmer en usage. L’instance
utilisateur est laissée ouverte et doit être relancée pour charger le nouveau binaire.


## Charsets directs 0.4.1 — 27 septembre 2026

- Mapping limité à 36 sources A–Z/0–9. Le routage accepte les dix positions de la
  rangée numérique sans Maj sur AZERTY, uniquement dans le canevas clavier actif.
- Braille : 66 motifs sélectionnés, en quatre hauteurs de 12/17/22/15 formes.
  A/Z/E gardent les colonnes gauche/droite/double alignées en bas. Les banques ont
  des plages explicites ; une touche libre ne déborde jamais sur la suivante.
- Les autres presets conservent leurs glyphes et leurs 26 mappings principaux.
  Le répertoire souris conserve les 665 glyphes et les 256 motifs braille.
- `cargo test --locked --quiet` : **72 réussis**, 1 test GPU ignoré par défaut.
  Les régressions vérifient sources, portée du routage AZERTY (modificateurs,
  IME, champs), hauteur exacte, épaisseurs, symétries, unicité et accessibilité
  des cibles ; le panneau compact affiche toutes les correspondances affectées.
- `cargo fmt --all --check` et `cargo clippy --locked --all-targets -- -D warnings` : OK.
- Bundle macOS 0.4.1 optimisé, signé ad hoc et vérifié avec `codesign --verify`.
- Le binaire du bundle a passé `--smoke-dir validation/runtime/direct-charsets/native`
  sous Apple M2 / Metal, avec SF Mono Regular : édition, routage de la rangée
  numérique, sauvegarde/rechargement V4, effets et export PNG.
- La capture `validation/runtime/direct-charsets/native/braille-window.png` et la
  planche des quatre hauteurs `validation/runtime/direct-charsets/braille.png`
  ont été inspectées visuellement. L’export du scénario correspond pixel par pixel
  au projet rechargé (640×800).
- Build Linux x86_64 0.4.1 réussi avec la toolchain croisée isolée, sans changement
  global : journal `validation/runtime/linux-build-v041.log`. Archive reconstruite
  avec les guides à jour. Aucune session graphique Linux n’a été exécutée.

Le scénario natif pilote le routage interne et capture le framebuffer réel ; il ne
constitue pas une vérification par frappes physiques sur un clavier Mac. L’ancienne
instance de l’utilisateur est laissée ouverte ; sauvegarder puis relancer le bundle
pour charger cette version. Le warning préexistant de `block 0.1.6` et un avertissement
Zig sur une option d’optimisation du linker obsolète n’empêchent pas les builds.


## Shaders 0.4.0 — 27 septembre 2026

Validé sur Apple M2 / Metal, SF Mono Regular. Le bundle final est
`/Users/mars/projet-perso/ditto/dist/Ditto.app` (0.4.0), signé ad hoc ;
`dist/Ditto-macOS-arm64.zip` a été reconstruit. Le smoke final a été exécuté
avec le binaire contenu dans ce bundle. Le binaire compilé et le binaire signé
ont le même UUID Mach-O ; la signature explique leur différence octet à octet.

- `cargo fmt --check` : OK.
- `cargo clippy --locked --all-targets -- -D warnings` : OK.
- `cargo test --locked` : 70 tests réussis, 1 test GPU ignoré par défaut.
- `cargo test --locked --test shaders -- --include-ignored` : 5 réussis,
  dont le test GPU (71 tests distincts validés au total).
- `cargo run --locked --example shader_proof` : les quatre looks, PNG transparent
  et projet V4 de démonstration sont produits et la réouverture est identique.
- `dist/Ditto.app/Contents/MacOS/ditto --smoke-dir validation/runtime/shaders/release-smoke` : OK,
  fenêtre native, rendu Metal, édition de pile, annuler/rétablir, sauvegarde,
  rechargement, export PNG et capture du framebuffer présenté.
- `codesign --verify --deep --strict --verbose=2 dist/Ditto.app` : OK.

Les régressions couvrent les limites et fichiers de look invalides, les projets
V1/V2/V3, la pile V4, la désactivation sans différence de pixels, le mélange nul,
la conservation des cellules et de l’historique sur une grande grille, les neuf
shaders exécutés, l’alpha des zones vides, les halos, l’ordre des passes, le grain
reproductible et mis à l’échelle, la composition opaque après effets et les exports
×1/×2/×4. Les tests UI couvrent les commandes, le clavier, le retour du sélecteur de
couleur et l’absence de contrôles superposés ou hors fenêtre à 1120×720 et 1184×832.

Preuves : `validation/runtime/shaders/release-smoke.log`,
`validation/runtime/shaders/release-smoke/runtime.txt`,
`validation/runtime/shaders/release-smoke/shaders-window.png`,
`validation/runtime/shaders/shader-demo.ditto` et `look-0.png` à `look-3.png`.
La capture finale a été inspectée visuellement ; les accents À/È ont été ajoutés
au seul atlas UI pour corriger les libellés. Le smoke pilote les commandes de
l’application et capture son vrai framebuffer. L’outil d’interaction macOS n’a
pas réussi à attacher une fenêtre Ditto (`cgWindowNotFound`) : cela ne constitue
pas une recette manuelle exhaustive à la souris ou VoiceOver.

Les shaders Linux et les très grandes grilles en usage prolongé restent à valider.
L’archive Linux déjà présente reste en 0.3.1. Aucun paquet tiers n’a été ajouté.
L’avertissement de compatibilité future de `block 0.1.6`, déjà présent, subsiste.
L’ancienne instance Ditto de l’utilisateur a été laissée ouverte ; elle doit être
relancée pour charger le binaire 0.4.0. Les seuls processus QA lancés pour cette
validation ont été arrêtés.


## Correction 0.3.1 — raccords sans interligne dans les caractères graphiques

Les blocs, ombrages et traits utilisent désormais leur géométrie de cellule sans
les marges typographiques de SF Mono. Leurs texels de bord sont prolongés dans les
gouttières de l’atlas afin d’éviter les lignes sombres dues au filtrage lors du zoom.
La police des lettres et les proportions 8×16 restent identiques.

62 tests réussis. Les nouveaux cas vérifient un pavé de 3×3 blocs sans aucun pixel
vide à échelle ×1/×2/×4, les raccords à fort zoom et l’absence de rangées/colonnes
vides dans les traits et ombrages. Le PNG `validation/runtime/line-spacing/jointive.png`
a été inspecté visuellement. Formatage, clippy strict et signature macOS vérifiés.

## Version 0.3 — SF Mono pour l’interface, le dessin et le PNG

- 59 tests réussis, incluant le chargement local de SF Mono Regular, l’antialiasing,
  la correspondance atlas GPU / alpha PNG, les gouttières transparentes, les glyphes
  de repli, les proportions des références et la lecture des projets V1/V2.
- Le chargement réel a identifié `SF Mono Regular` depuis
  `/Library/Fonts/SF-Mono-Regular.otf`. Le fichier n’est pas inclus dans les archives.
- Le PNG de contrôle `validation/runtime/sfmono/sfmono.png` a été inspecté visuellement :
  lettres, chiffres, accents, traits, blocs, braille et sextants.
- Les bitmaps de repli historiques restent inchangés ; le rendu principal et la taille
  des PNG changent volontairement. Les anciennes comparaisons pixel V1/V2 documentées
  ci-dessous concernent les versions antérieures à ce changement de police.
- Le scénario natif Metal 0.3 passe et identifie SF Mono Regular. Le PNG exporté
  correspond pixel par pixel au projet rechargé, aux nouvelles dimensions 640×800.
- La limite d’automatisation de capture macOS décrite pour 0.2 reste applicable.

## Extension 0.2 : mode clavier et charsets

La version 0.2 ajoute le basculement souris/clavier, six charsets, 79 touches sources
par banque et le répertoire de 665 glyphes. La catégorie Blocs comporte 397 entrées.
Le guide est dans [CLAVIER-CHARSETS.md](docs/CLAVIER-CHARSETS.md).

- **53 tests réussis** : 31 états/interactions/routage clavier, 13 tests historiques
  cœur/fichiers/PNG, 9 tests de mapping, géométrie, atlas et compatibilité V1/V2.
- `cargo fmt --check` et `cargo clippy --locked --all-targets -- -D warnings` passent.
- Build macOS arm64 optimisé et signature ad hoc vérifiée.
- Build Linux x86_64 optimisé réussi avec la même toolchain croisée isolée que la V1.
- Le scénario natif interne sous **Apple M2 / Metal** passe avec le mode clavier,
  les lettres/chiffres/signes, une banque braille, la sauvegarde V2, la réouverture
  et l’export PNG. Il vérifie l’avancement du curseur et la présence d’indices étendus.
- Le PNG du scénario V2 correspond pixel par pixel au projet rouvert.
- L’ancien projet V1 de recette et son PNG restent identiques pixel par pixel avec
  le moteur 0.2. Les bitmaps des 256 glyphes historiques sont également comparés
  exhaustivement aux octets originaux dans les tests.
- L’export des 79 blocs/ombrages du preset a été inspecté visuellement.

**Limite de recette actuelle :** l’utilisateur a confirmé que la fenêtre de test
était visible, mais l’outil de contrôle natif n’a pas pu s’y attacher. Après
`cgWindowNotFound`, ScreenCaptureKit a renvoyé l’erreur `-3811` (« audio/video capture
failure »), y compris après réinitialisation de l’outil. La frappe réelle dans la
nouvelle interface n’a donc pas été automatisée ni certifiée dans cette session.
Les tests de routage utilisent des événements clavier simulés dans le code de
l’application ; ils ne sont pas présentés comme des frappes envoyées au système.
Aucun réglage de sécurité ou service macOS n’a été modifié pour contourner l’erreur.

Preuves locales de cette extension :

- `validation/runtime/charsets-v02/catalogue.txt` : décompte des répertoires et banques.
- `validation/runtime/charsets-v02/blocs.ditto`, `blocs.png`, `blocs.txt` : export du preset complet.
- `validation/runtime/charsets-v02/native/` : projet V2, PNG, texte et résultat du scénario Metal.
- `validation/runtime/charsets-v02/native.log` : `SMOKE OK` et adaptateur Metal.
- `validation/runtime/linux-build-v02.log` : compilation Linux de la version 0.2.

Commandes de reproduction :

```sh
cargo test --locked
cargo run --quiet --example charset_proof -- validation/runtime/charsets-v02
target/release/ditto --smoke-dir validation/runtime/charsets-v02/native
cargo run --quiet --example verify_export -- \
  validation/runtime/charsets-v02/native/smoke.ditto \
  validation/runtime/charsets-v02/native/smoke.png
```

Les sections ci-dessous conservent l’historique de la recette V1.

## Environnement et périmètre

- Workspace : `/Users/mars/projet-perso/ditto`.
- Base Git : non applicable ; le dossier initial n’était pas un dépôt Git.
- Hôte testé : macOS 27.0 (26A428), Apple Silicon arm64, Apple M2.
- Rendu réellement initialisé : `Apple M2 / Metal`.
- Compilateur natif : Rust/Cargo 1.97.0 Homebrew ; dépendances dans `Cargo.lock`.
- Aucun push, publication ou déploiement distant. Le workflow GitHub fourni n’a pas été exécuté à distance.

## Contrôles automatisés

| Contrôle | Résultat |
| --- | --- |
| `cargo fmt --check` | Réussi |
| `cargo clippy --locked --all-targets -- -D warnings` | Réussi |
| `cargo test --locked` | 25 tests réussis : 12 interactions/état et 13 cœur/fichiers/rendu |
| Compilation optimisée macOS | Réussie |
| `codesign --verify --deep --strict dist/Ditto.app` | Réussi, signature ad hoc |
| `plutil -lint dist/Ditto.app/Contents/Info.plist` | Réussi |
| Scénario natif `--smoke-dir` | Réussi sur Metal avec fenêtre réelle |

Les tests discriminants couvrent notamment les trajets souris interpolés, l’équivalence
souris/clavier des rectangles, les déplacements avec chevauchement, les masques du
pinceau, le remplissage borné par sélection, les annulations et branches d’historique,
la conservation de l’état modifié après annulation d’une navigation, la suppression limitée à la cellule courante, le recadrage réversible, les accents composés, le texte incompatible, les écritures
échouées, les projets déplacés avec référence embarquée, la transparence, les exports
à échelle entière et la conservation d’un brouillon non encore restauré.

La fenêtre compacte 1120 × 720 fait l’objet d’un contrôle des zones interactives :
les commandes restent dans les limites de la fenêtre et la grille de glyphes est paginée.
Ce contrôle de géométrie ne remplace pas une recette de redimensionnement sur tous les OS.

Un avertissement de compatibilité future provient de la dépendance transitive
`block 0.1.6`. Il ne bloque pas les contrôles avec le compilateur utilisé ; le graphe
est verrouillé. Les migrations de toolchain devront refaire les vérifications.

## Parcours macOS réellement exécuté

La recette a utilisé un bundle séparé `validation/runtime/Ditto QA.app`, avec un
identifiant distinct et un brouillon isolé. La session initiale ouverte par l’utilisateur
n’a pas été fermée ni remplacée en cours d’édition.

1. Ouvrir le choix rapide et créer un sprite 32 × 32.
2. Activer Texte, saisir réellement `café`, ouvrir l’aperçu de texte et retrouver
   exactement ces caractères. Le projet enregistré contient les glyphes `[99, 97, 102, 130]`.
3. Dessiner un rectangle au clavier en choisissant ses deux points.
4. Tracer à la souris dans le canevas natif.
5. Importer une image via le sélecteur de fichiers macOS et retrouver ses commandes
   de visibilité, verrouillage, opacité et ajustement.
6. Enregistrer via le dialogue macOS ; le projet contient `manifest.json`,
   `drawing.json` et `assets/reference.png`.
7. Copier le projet dans un autre dossier, le rouvrir dans la version optimisée et
   retrouver le dessin ainsi que sa référence.
8. Exporter un PNG via les dialogues de l’application et du système.
9. Comparer les pixels du PNG à la composition du projet rouvert : égalité exacte,
   dimensions 512 × 512, référence embarquée mais absente du PNG.

Le rendu et les dialogues ont été inspectés dans la fenêtre native. Les captures
correspondantes figurent dans la conversation de travail ; elles ne sont pas présentées
comme des captures de la maquette HTML.

Les API d’accessibilité exposent les boutons, les dimensions éditables, les choix de
palette, le canevas et ses coordonnées. Cela a permis d’actionner réellement le choix
de format, les outils et les exports. Une recette VoiceOver exhaustive reste à faire.

## Preuves locales

Ces données sont des artefacts de recette et sont exclues du suivi par `.gitignore` :

- `validation/runtime/moved/native-final.ditto` : projet rouvert après déplacement.
- `validation/runtime/smoke/native-final.png` : PNG exporté par l’interface native.
- `validation/runtime/release-smoke/` : projet, PNG, texte et résultat du scénario automatisé natif.
- `validation/runtime/release-smoke.log` : sortie Metal et mesure `/usr/bin/time -l`.
- `validation/runtime/session-before-qa.ditto` : copie de précaution du dessin de la
  première session, réalisée avant les opérations de recette.

Vérification reproductible de la sortie native :

```sh
cargo run --quiet --example verify_export -- \
  validation/runtime/moved/native-final.ditto \
  validation/runtime/smoke/native-final.png
```

Résultat obtenu : `PNG pixels match reopened project: 512 × 512; reference embedded: true`.

## Mesure courte de la version optimisée

Commande :

```sh
/usr/bin/time -l target/release/ditto --smoke-dir validation/runtime/release-smoke
```

Sur cet hôte, le processus complet du scénario court a duré 0,88 seconde, avec
97 583 104 octets de mémoire résidente maximale (environ 93,1 Mio) et 83 297 216 octets
pour l’indicateur macOS « peak memory footprint ».

Ce n’est pas une mesure isolée du temps d’ouverture, ni une garantie de mémoire en
édition intensive. Le profilage prolongé sur des grilles 512 × 512, de gros remplissages
et des références maximales reste à faire.

## Linux

Le chemin de compilation `x86_64-unknown-linux-gnu` a passé `cargo check --locked` avec
un compilateur Rust officiel isolé dans `/tmp/ditto-linux-sysroot` et la bibliothèque
standard Linux correspondante. Le journal est dans `validation/runtime/linux-check.log`.

La première tentative avec le compilateur Homebrew et la bibliothèque standard officielle
a été rejetée par Rust pour incompatibilité de métadonnées. Le contrôle a été refait avec
le compilateur et les bibliothèques officiels concordants, sans modifier la toolchain système.

Un binaire ELF x86_64 optimisé a également été produit :
`target/x86_64-unknown-linux-gnu/release/ditto`. L’édition de liens a réussi avec
Zig 0.16.0 ciblant `x86_64-linux-gnu.2.35` et le compilateur Rust officiel 1.97.0.
La limite de descripteurs a été portée à 4096 uniquement dans le processus de build,
après un premier échec `ProcessFdQuotaExceeded`. Le log final est dans
`validation/runtime/linux-build.log`.

Commande de build exécutée après préparation de la toolchain isolée :

```sh
ulimit -n 4096
RUSTC=/tmp/ditto-linux-sysroot/bin/rustc \
RUSTFLAGS='--sysroot /tmp/ditto-linux-sysroot' \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/ditto-cross/linux-linker \
cargo build --release --locked --target x86_64-unknown-linux-gnu
```

Le linker temporaire invoque `zig cc -target x86_64-linux-gnu.2.35`. Les archives
Rust et Zig officielles téléchargées ont été vérifiées avec leurs SHA-256 publiés.
Aucun changement de toolchain globale ou de configuration du shell n’a été effectué.
L’archive livrable est `dist/ditto-linux-x86_64.tar.gz`.
L’affichage, les entrées, le presse-papiers et les portails Wayland/X11 n’ont pas été
exécutés : Docker Desktop ne démarrait pas et aucune session graphique Linux n’était
accessible. Ne pas présenter le contrôle de compilation comme une recette Linux.

## Limites de livraison

- `.app` arm64 local, signé ad hoc, non notarisé ; pas de test Intel ou ancien macOS.
- Un plan de dessin, une référence, un document actif.
- Pas d’association Finder au double-clic ; ouvrir dans l’application ou par CLI.
- Pas de conversion automatique d’image, d’animation, d’import `.xp` ou d’ANSI coloré.
- Police UI limitée au répertoire décrit dans le README ; alphabets arbitraires non rendus.
- Historique non persistant ; texte annulé par événement de saisie, sans regroupement temporel.
- Pas de promesse de comportement du presse-papiers après fermeture sur chaque environnement Linux.
- La validation Linux graphique et la recette complète des lecteurs d’écran restent ouvertes.
