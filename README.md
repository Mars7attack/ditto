# Ditto 0.5.0

Atelier de caractères natif pour macOS et Linux, écrit en Rust avec winit et wgpu.
L’intérieur de la fenêtre est une interface terminal : SF Mono, cadres,
commandes textuelles et couleurs sobres inspirées du cadrage Lain/rétro.

## Lancer

Sur le Mac de développement : ouvrir `dist/Ditto.app`.

Les archives macOS et Linux de `dist/` sont en version 0.5.0.

Sous Linux x86_64 : extraire `dist/ditto-linux-x86_64.tar.gz`, puis lancer
`./ditto` depuis le dossier extrait. Le binaire est compilé pour glibc 2.35
ou ultérieur ; son utilisation graphique reste à valider sur Linux.

Depuis les sources, avec Rust 1.88 ou ultérieur :

```sh
cargo run --release --locked
cargo run --release --locked -- chemin/du/projet.ditto
```

La première ouverture propose un choix rapide : sprite 32 × 32, illustration
80 × 50 ou carte 100 × 60. Les dimensions restent personnalisables.

## Police SF Mono

SF Mono Regular est utilisée pour l’interface **et le dessin**, ainsi que les PNG.
Les traits, blocs et ombrages se raccordent bord à bord, sans bandes vides entre
les lignes, y compris dans les PNG. Leur géométrie est adaptée à la cellule ;
le texte garde SF Mono. Le rendu est lissé ; les cellules ont les proportions naturelles 8 × 16, avec
une grille logique inchangée. Les glyphes absents de la police disposent d’un
repli géométrique, notamment les sextants et le braille.

La police est chargée localement sur macOS, sans fichier Apple incorporé dans
l’application ou les projets. Sous Linux, Ditto utilise SF Mono si elle est
installée dans les chemins habituels, sinon DejaVu Sans Mono ou une autre
monospace locale. `DITTO_FONT_PATH` permet d’indiquer un fichier local.

Les projets V1/V2/V3/V4 restent lisibles ; ils s’affichent désormais avec la nouvelle
police. Les sauvegardes sont au format V5. Les positions/couleurs/caractères
sont préservés, mais le rendu et les proportions changent volontairement.
Pour réajuster une ancienne référence aux nouvelles proportions, utiliser
« Contenir » ou « Remplir ».

## Mode clavier et charsets

Le bouton **Mode : souris / clavier** dans la barre supérieure active la saisie inspirée
d’Asciitor. Le panneau des glyphes devient un sélecteur de charset avec correspondances
visibles. Une frappe insère le glyphe et avance. Six charsets utilisent uniquement les
36 touches A–Z et 0–9 ; la rangée des chiffres fonctionne sans Maj sur AZERTY.
Le braille propose 148 formes : quatre banques par hauteur et deux pages
**Espacés / lignes** et **Espacés / blocs**, qui regroupent les motifs avec des rangées vides. Les autres banques gardent leurs glyphes.
**Cmd + ↑ / ↓** change de banque sur MacBook, sans Fn (Ctrl + ↑ / ↓ sous Linux).

Le répertoire compte 665 glyphes et la catégorie Blocs comporte 397 entrées, dont
fractions, quadrants, textures, sextants et braille. [Guide détaillé](docs/CLAVIER-CHARSETS.md).

## Dessiner

- Crayon, gomme, ligne, rectangle vide/plein, remplissage, texte, sélection et pipette.
- 665 glyphes : CP437 historique, traits, blocs, ombrages, sextants et braille.
- Glyphe, premier plan et fond s’appliquent indépendamment via les cases du pinceau.
- Clic sur une couleur : premier plan. Clic droit : fond. Shift-clic : modifier la palette.
- Cliquer sur FG/BG pour saisir une couleur hexadécimale. « Fond vide » applique un fond transparent.
- Sélectionner une zone puis la déplacer, la couper, la copier ou l’effacer.
- Coller produit un bloc flottant ; Entrée confirme et Échap annule. Repositionner un bloc qui dépasse la grille.
- Un trait continu correspond à une seule action annulable. La perte de focus annule un geste encore provisoire.

Le dessin et l’interface ont des grilles indépendantes. Le zoom du canevas ne
grossit pas les menus. Le zoom au trackpad suit progressivement la distance parcourue ;
la molette change l’échelle d’environ 5 % par cran. Le point sous le pointeur reste fixe. Le répertoire étendu est paginé ; les catégories
Traits/Blocs/Récents restent accessibles en mode souris.

## Guides, recoloration et thème

- **D / Guides** : tracer librement sur un calque transparent au-dessus du canevas.
  **Guides F8** règle sa couleur, son épaisseur, son opacité et sa visibilité,
  avec une gomme de traits. Le calque est sauvegardé mais exclu des exports.
- **C / Recolorer** : repeindre uniquement la couleur FG des caractères existants,
  sans modifier les glyphes ou leur fond. La brosse respecte la sélection ;
  sa taille se règle avec − / + ou **[ / ]**. Chaque geste est annulable.
- **Réglages / Cmd-Ctrl + virgule** : thèmes Ditto, Minuit et Papier, et douze
  couleurs d’interface personnalisables. Aperçu immédiat, annulation et
  sauvegarde locale des préférences, indépendamment du dessin.

[Utilisation, architecture et format V5](docs/OUTILS-ET-REGLAGES.md).

## Clavier

Les commandes documentaires utilisent Cmd sur macOS et Ctrl sur Linux.
Selon le réglage du clavier Mac, les touches F1–F6 peuvent nécessiter Fn.

| Commande | Action |
| --- | --- |
| Cmd/Ctrl Maj-M | Basculer souris / clavier |
| B / G / L / R (mode souris) | Crayon / gomme / ligne / rectangle |
| C / D (mode souris) | Recolorer / tracer des guides |
| F8 / Cmd-Ctrl + virgule | Calque de guides / réglages de l’application |
| F / I / T / M (mode souris) | Remplissage / pipette / texte / sélection |
| Flèches | Déplacer le curseur ou parcourir la palette ayant le focus |
| Entrée | Appliquer une cellule ; poser puis confirmer les deux points d’une forme |
| Échap | Annuler le geste, sortir de la saisie ou fermer le dialogue |
| F3 / F4 / F5 / F6 | Glyphes ou charset / couleurs / outils / canevas |
| Page précédente/suivante (clavier) | Changer de banque |
| Tab / Shift-Tab | Parcourir les contrôles, sans traverser chaque case d’une palette |
| Shift-Entrée sur une couleur | Modifier la couleur de palette |
| Espace + glisser / Alt + flèches | Déplacer la vue |
| Molette / + / - / 0 | Zoom / agrandir / réduire / ajuster |
| Cmd/Ctrl N / O / S / Shift-S | Nouveau / ouvrir / enregistrer / enregistrer sous |
| Cmd/Ctrl Z / Shift-Z | Annuler / rétablir |
| Cmd/Ctrl C / X / V | Copier / couper / coller les cellules |
| Cmd/Ctrl Shift-C | Aperçu du texte à copier |
| Cmd/Ctrl A / Suppr | Tout sélectionner / effacer la sélection |
| Cmd/Ctrl E | Export PNG |
| F1 / F2 | Aide / redimensionner |
| F7 | Ouvrir la fenêtre Shaders |

En mode Texte, les caractères remplacent les cellules ; ils ne décalent pas
le dessin. Les accents composés sont normalisés. Le texte hors du répertoire
ou des dimensions disponibles est signalé sans insertion partielle. Les
raccourcis d’outils sont inactifs pendant la saisie.

## Shaders

Ouvrir **[Shaders F7]** pour travailler le rendu sans modifier les cellules.
La fenêtre garde l’interface terminal de Ditto et propose un aperçu sur damier,
un bouton **Comparer avant/après**, un catalogue et une pile de huit effets maximum.
L’aperçu se zoome à la molette/au trackpad et se déplace en glissant. **− / +**,
**Ajuster** et **100 %** permettent de régler le cadrage sans fermer le panneau.

- **Blur** : flou global, avec rayon et composantes horizontale/verticale.
- **Blur des contours** : flou sélectif des transitions, avec rayon, seuil et douceur.
- **Glow** : halo lumineux, rayon, puissance et seuil.
- **Brillance** : reflet oblique avec position, largeur et éclat.
- **Couleurs** : teinte, saturation et exposition.
- **Duotone** : deux encres modifiables en hexadécimal, contraste et inversion.
- **Scanlines** : espacement, épaisseur et obscurité des lignes.
- **Motif** : points, lignes diagonales ou damier, avec densité et espacement.
- **Chromatique** : séparation rouge/bleu, distance, angle et balance.
- **Grain** : bruit fixe, taille, intensité et graine reproductible.
- **Vignette** : rayon, douceur et obscurité des bords.

Les effets sont appliqués de haut en bas. Sélectionner un effet pour régler son
mélange et ses paramètres avec **− / +** ; utiliser **↑ / ↓** pour le déplacer,
sa case pour le désactiver, ou **Retirer**. Le bouton **Effets** désactive toute
la pile. **Comparer avant/après** agit seulement sur l’aperçu et ne modifie pas
le projet ni l’export. Les changements s’annulent avec Cmd/Ctrl Z et se rétablissent
avec Cmd/Ctrl Shift Z. Tab parcourt les contrôles ; Entrée les active.

Quatre looks sont fournis : **Néon, CRT, Riso, Irisé**. Un look remplace la pile,
ce qui est annulable. **Sauver look / Charger look** échange des recettes locales
`.ditto-shaders`. La pile fait aussi partie du projet `.ditto` et de sa récupération.
**Fermer** ou Échap revient au dessin en conservant les réglages.

Les PNG incluent les shaders actifs ; le fond opaque éventuel est ajouté après
les effets. Le halo peut occuper les pixels transparents autour des glyphes, mais
reste dans les limites de la grille. Le texte copié reste constitué des caractères
originaux. Les effets sont fixes : pas de timeline ni d’import de code WGSL dans
cette version. [Guide et architecture](docs/SHADERS.md).

## Référence

Importer un PNG/JPEG par « Importer image » ou glisser-déposer. L’image est
copiée dans le projet. Opacité, visibilité, centrage, ajustement et échelle
sont réglables. Déverrouiller, puis activer « Déplacer » pour la manipuler
avec la souris ou les flèches. Une référence verrouillée reste fixe.

« Tracé » atténue temporairement le dessin pour voir la référence derrière
les fonds opaques. Cette aide n’affecte ni le document ni les exports.

## Sauvegarde et sorties

Le `.ditto` contient un manifeste, les cellules, le calque de guides et la référence embarquée. Les anciens
projets restent lisibles ; les nouvelles sauvegardes utilisent le format V5.
Une écriture temporaire suivie d’un remplacement atomique protège le fichier
précédent en cas d’échec. Un brouillon de récupération est créé après une
pause d’édition, séparément du projet. Il est proposé à la prochaine ouverture.

- **Texte** : sélection ou grille entière, UTF-8, avec espaces d’alignement,
  sans couleurs. Sa présentation dépend de la police de l’application de destination.
- **PNG** : grille entière, cellules SF Mono de 8 × 16 pixels, échelle ×1/×2/×4,
  transparence ou fond opaque. Ni référence ni aides d’édition.

Les indices de glyphes sont conservés par le projet natif. La copie de cellules
interne conserve aussi les couleurs ; la copie explicite de texte sert aux
applications externes.

## Architecture

Les événements de fenêtre sont convertis en commandes. Le cœur Rust applique
ces commandes à la grille et conserve des différences pour l’historique. Les
changements structurels utilisent des instantanés partagés et bornés en mémoire.
Le rendu GPU reçoit les cellules visibles, un atlas antialiasé de la police locale
et la référence. L’export PNG compose les mêmes masques et couvertures côté CPU ;
il ne capture pas la fenêtre. Quand une pile de shaders est active, le dessin
rasterisé traverse des passes WGSL sur trois textures intermédiaires. Ce même
pipeline est utilisé pour l’aperçu et pour le PNG, avec une échelle exprimée en
pixels du document. Le dessin et le résultat sont mis en cache pendant la navigation ;
les effets ne déclenchent aucune animation ni boucle de rendu au repos.

AccessKit expose les commandes, les champs, le canevas et les états aux API
d’accessibilité. Le décodage, la sauvegarde et l’export s’exécutent en arrière-plan.
Au repos, l’application attend les événements au lieu de redessiner continuellement.

## Construire les livrables

```sh
scripts/package-macos.sh
# Sur une machine Linux :
scripts/package-linux.sh
```

Le bundle macOS est signé ad hoc pour l’usage local ; il n’est pas notarisé.
Le script Linux produit une archive contenant l’exécutable et sa documentation.
Sur Linux, prévoir les bibliothèques de fenêtre et un pilote GPU compatible,
ainsi qu’un portail de fichiers compatible ou Zenity pour les dialogues.

La CI locale fournie décrit des builds macOS et Ubuntu. Elle n’a pas été publiée
ni exécutée sur un service distant dans cette session.

## Vérification

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
# Sur une machine avec GPU :
cargo test --locked --test shaders -- --include-ignored
cargo run --release --locked -- --smoke-dir validation/runtime/smoke
```

Le dernier scénario ouvre une vraie fenêtre GPU, exerce l’édition, sauvegarde,
rouvre et exporte dans le dossier indiqué, puis quitte. Il isole son brouillon.
Les preuves et limites de la recette sont détaillées dans [VALIDATION.md](VALIDATION.md).

## Limites de cette V1

- Un seul document actif, un plan de dessin et une image de référence.
- Grille : 512 × 512 cellules maximum. Référence : 2048 × 2048 pixels et 32 Mio
  de source maximum. Export : 64 millions de pixels maximum. Historique : budget
  de 64 Mio, non conservé entre les sessions.
- L’ajustement à la vue peut être fractionnaire ; le PNG conserve l’échelle entière.
- Les caractères du catalogue absents de SF Mono utilisent un repli ; les libellés
  hors du répertoire UI peuvent afficher un caractère de remplacement. Les chemins
  de fichiers restent traités par le système sans conversion CP437.
- L’ouverture des projets se fait par « Ouvrir », glisser-déposer ou argument CLI.
  L’association Finder au double-clic n’est pas installée.
- Les lecteurs d’écran macOS disposent d’un arbre de contrôles ; une recette VoiceOver
  complète et la validation AT-SPI Linux restent à réaliser.
- La vérification de code et l’édition de liens Linux x86_64 ont réussi en compilation croisée.
  L’utilisation graphique Wayland/X11 reste non testée ; voir le rapport de validation.

Les extensions différées sont consignées dans la [spécification](SPEC-TECHNIQUE.md).
La police bitmap de repli Modern DOS provient de PC Face ; [attribution et licence](assets/README.md).
