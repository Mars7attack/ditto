# Validation automatisée

`scripts/check.sh` exécute formatage, Clippy strict et tous les tests sans fenêtre.
`scripts/check.sh --native` inclut les tests GPU, construit le binaire et
exécute deux parcours avec une vraie fenêtre winit/wgpu. Sur Linux sans bureau :
`xvfb-run -a -s '-screen 0 1920x1080x24' scripts/check.sh --native` avec Mesa Vulkan et `libxkbcommon-x11-0` (clavier X11 chargé dynamiquement).
Le workflow CI exécute la suite sans fenêtre sur macOS et Ubuntu et la suite
native sur Ubuntu/Xvfb. Aucun échec GPU n’est transformé en succès ou ignoré dans
le job natif.

## Niveaux de contrôle

1. **Contrats de données** : géométrie, masques, limites, migrations, sérialisation,
   exports, recettes de shaders, couleurs et historique. Balayages déterministes
   des entrées et oracle de snapshots indépendant du stockage delta de l’historique.
2. **Intégration d’édition** : routage des touches, coordonnées des contrôles,
   glissements, focus, modales, undo/redo et tâches de sauvegarde/récupération.
   La présence du curseur est vérifiée dans la géométrie réellement émise par l’UI.
3. **Parcours natifs de bout en bout dans l’application** : entrées → état → UI →
   GPU → captures, projet sur disque → réouverture → export. Assertions de pixels
   ciblées plutôt que comparaison d’une fenêtre entière sensible aux polices/DPI.
   Un second parcours compare les contours des glyphes avec un shader neutre.
   La grille est aussi rendue dans des textures à cinq résolutions physiques,
   via le même pipeline que la fenêtre, puis contrôlée pixel par pixel.

## Matrice de comportements

| Domaine | Cas automatisés | Suites |
|---|---|---|
| Crayon, gomme, ligne, rectangle | Interpolation, tous les octants, formes pleines/contours, bords dégénérés, 8 masques, sélection, geste unique, annulation à la perte de focus | `core_and_project`, `behavior_contracts`, tests d’application |
| Remplissage, texte, pipette | Limites par canaux/sélection, prélèvement, saisie, suppression, fin de ligne/document, normalisation Unicode, rejet atomique des caractères non supportés | Tests d’application et de routage, `behavior_contracts` |
| Sélection, déplacement, collage | Sélection inversée, recouvrement, collage exact/hors grille, espaces finaux, validation/annulation et undo | `core_and_project`, `behavior_contracts`, tests d’application |
| Mode clavier / charsets | AZERTY/chiffres, modificateurs/AltGr, IME, banques, mappages, répertoire étendu, navigation et aller-retour souris/clavier | `extended_glyphs`, tests de routage et d’édition |
| Curseur | Chacun des 11 outils souris, 8 cycles par outil, couleur de guide validée/annulée, fermeture de chaque panneau, focus, IME, sélection, trois thèmes, guides devant/derrière, shaders, zoom, pan, bords et redimensionnement 1×1 | `regression_tests`, captures natives `cursor-*` |
| Indépendance clavier/souris | Même séquence de saisie, sélection, suppression et historique pour les 11 outils mémorisés ; Échap préserve l’outil souris et efface la sélection clavier ; IME avec focus UI ; undo/redo après navigation hors vue ; quatre côtés du cadre visibles au bord du viewport | `regression_tests`, six captures natives `cursor-guide-stroke-keyboard` à `cursor-text-tool-keyboard-escape` |
| Couleurs | Tous les types de cible, RGB↔HSV sur un cube de 4096 couleurs et les 256 gris, noir/blanc, conservation de teinte, drags hors limites, perte de focus, invalidité HEX, clavier, annulation, palette/shader undo, layout minimum, arbre d’accessibilité | `color_picker`, `regression_tests`, captures natives `picker-*` |
| Palette liée à la brosse | Picker FG, swatches dupliqués, modification d’une autre case, pipette indépendante, annulation/rétablissement et persistance | `regression_tests`, captures natives `palette-*` |
| Sliders des effets | 44 réglages, aperçu pendant le geste, limites, valeurs discrètes, un undo par glissement, Échap/perte de focus, changement de couche, clavier, accessibilité et disposition minimum | `regression_tests`, captures `chromatic-slider-*` |
| Chromatique | Primaires sombres sans silhouettes noires, alpha partiel et nul, interpolation linéaire, sous-pixel, neutralité, échelles 1/2/4 et angles | Deux tests GPU dédiés, fixtures PNG/captures natives |
| Partage texte | UTF-8 exact, sélection, espaces/lignes vides, HTML échappé, clôtures Markdown, découpe Discord aux lignes et budget UTF-16, erreur explicite, aperçu du bloc, zoom/pan et fichier .txt | `text_export`, `regression_tests`, captures `text-export-*` et fichiers natifs |
| Géométrie du braille exporté | Tous les 256 motifs, blancs U+2800, opt-out source exact, texte mixte non converti, budget UTF-16 inchangé, option UI sans mutation du dessin, mêmes sorties fichier/copier ; avances et positions mesurées par CoreText sur macOS | `text_export`, `regression_tests`, `text-export-braille-*`, scripts de diagnostic |
| Guides | Sous-cellule, coupure à la sortie du canevas, gomme balayée, visibilité, ordre, opacité/épaisseur limites, couleur future vs recoloration globale, suppression, projet et export sans aides | `guides_and_settings`, tests d’application, composition native vérifiée en pixels |
| Recoloration | FG seul, espaces et braille vide ignorés, masque indépendant, sélection, rayon borné, interpolation, sortie/rentrée, undo du geste | `guides_and_settings`, tests d’application, parcours natif |
| Référence | Import réel PNG, transformations, verrouillage, fit/cover, opacité bornée, suppression/undo, portabilité, exclusion des exports | `core_and_project`, `sfmono`, `regression_tests`, composition native |
| Shaders | Les 11 effets, paramètres aux extrêmes, pile limitée/réordonnée/supprimée, activation, presets invalides, avant/après, navigation aperçu indépendante, alpha, échelle, glow et flous, export cohérent | `shaders`, `behavior_contracts`, tests d’application, tests GPU et parcours de qualité |
| Thème | Presets, 12 tokens, brouillon/annulation/enregistrement, isolation du document, échec de persistance sans perte de préférences | `guides_and_settings`, tests d’application, captures natives |
| Historique | Undo/redo, annulation, branche après undo, no-op conservant redo, 160 opérations mixtes comparées à des snapshots, état dirty et données partagées | `core_and_project`, `behavior_contracts`, tests d’application |
| Fichiers / récupération | Save/load V1–V6, version inconnue et corruption, entrées absentes/troncature ZIP, document invalide préservant le fichier existant, erreurs disque, édition pendant sauvegarde async, autosave/récupération isolée | `core_and_project`, `extended_glyphs`, `sfmono`, `guides_and_settings`, `behavior_contracts`, `regression_tests` |
| Grille / écrans | Chaque ligne couvre un pixel physique, DPI 1/1.25/1.5/2/3, zooms entiers/fractionnaires, pan et clipping, aller-retour entre densités sans dérive, grille masquée/sous le seuil | `render::grid_tests`, 30 captures GPU `editor/grid/` |
| Rendu / export | Atlas/contours, coutures, fallback, échelles/alpha, transparence, shaders, guides/référence exclus, pixel du curseur et couleur picker, roundtrip | `sfmono`, `shaders`, `core_and_project`, GPU et parcours natifs |
| Accessibilité / disposition | Navigation Tab, retours F6, champs, teinte ajustable, valeurs accessibles, contrôles sans chevauchement à la taille minimum | Tests d’application et `regression_tests` |

## Preuves et limites

Le lanceur natif crée un dossier unique sous `validation/runtime/native/`. Il
conserve les captures PNG, sondes de pixels JSON, projets, exports, rapports et
journaux même en cas d’échec ; il termine son propre processus après 120 secondes
en cas de blocage. Préférences et récupération sont isolées des fichiers utilisateur.
Les mêmes preuves sont jointes au job GitHub Actions.

Les parcours injectent les entrées dans le routage de l’application : ils ne
simulent pas un clavier physique au niveau du système. Les dialogues Finder/
portail, le presse-papiers partagé avec d’autres applications, les lecteurs
d’écran VoiceOver/AT-SPI complets, les IME système et Wayland nécessitent encore
une recette spécifique sur ces environnements. Les erreurs disque sont simulées,
pas une panne électrique. La matrice couvre les familles de fonctionnalités et
leurs principales frontières ; elle n’est pas une garantie de toutes les
combinaisons possibles ni une mesure de 100 % des lignes exécutées.

Pour une nouvelle fonctionnalité : ajouter son contrat, une régression qui échoue
sans la correction, et un scénario natif si le changement touche le rendu ou la
relation entre input/focus et canevas. Éviter les tests qui ne vérifient que le
nom d’une action ou recopient une condition interne.
