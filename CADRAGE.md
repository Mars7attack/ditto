# Ditto — recherche REXPaint et pré-spécifications

Date : 27 septembre 2026. « Ditto » est le nom de travail issu du dossier, à confirmer.

Statut : cadrage révisé après les réponses utilisateur du 27 septembre 2026. Les choix confirmés sont distingués des propositions de conception. La V1 Rust est maintenant implémentée ; consulter [README.md](README.md) et [VALIDATION.md](VALIDATION.md) pour l’utilisation et les preuves. La maquette reste une référence de conception.

La [spécification technique V1](SPEC-TECHNIQUE.md) rassemble désormais l’architecture, les contrats d’édition, le modèle documentaire, les exports et la recette. Ce cadrage conserve la recherche et le contexte produit ; consulter la spécification pour les précisions techniques et les arbitrages encore ouverts.

## 1. Ce que propose REXPaint

La page officielle propose actuellement REXPaint 1.70, daté du 10 janvier 2024, pour Windows. Wine est indiqué comme possibilité sur Linux/macOS ; cela ne constitue pas une distribution native. Les indications macOS de cette page sont anciennes et leur fonctionnement actuel n'a pas été testé. [Téléchargements](https://www.gridsagegames.com/rexpaint/downloads.html)

REXPaint sert au dessin de caractères, à l'art ANSI et à la préparation de cartes, interfaces ou ressources de jeux. Son intérêt pour notre projet est de traiter les caractères comme une matière graphique dans une grille. [Présentation](https://www.gridsagegames.com/rexpaint/index.html)

| Famille | Fonctions documentées |
| --- | --- |
| Cellules | Caractère, couleur du caractère et couleur du fond modifiables séparément. |
| Dessin | Tracé libre, lignes, rectangles, ovales, remplissage, texte ; raccordement automatique des traits pour les cadres et murs. |
| Manipulation | Sélection rectangulaire, couper/copier/coller, retournements et transformations ; annuler/rétablir. |
| Confort | Prévisualisation sous le curseur, déplacement du canevas, redimensionnement, changement d'échelle par les polices. |
| Couleurs | Sélecteur RGB/HSV/hexadécimal, palettes, recoloration et échanges de couleurs. |
| Composition | Calques, visibilité, verrouillage, ordre et fusion. |
| Ressources | Polices bitmap, caractères étendus, jeux de tuiles, thèmes d'interface, navigateur d'images. |
| Échanges | Projet `.xp`, PNG, TXT, ANSI `.ans`, CSV, XML, XPM, BBCode et C:DDA ; import TXT et ANSI. |

Sources : [fonctions et démonstrations](https://www.gridsagegames.com/rexpaint/features.html), [manuel officiel](https://www.gridsagegames.com/rexpaint/manual.txt), [ressources](https://www.gridsagegames.com/rexpaint/resources.html).

### Détails qui influencent notre conception

- Les calques sont limités à neuf ; le fond magenta sert de marqueur de transparence.
- Le format `.xp` conserve des indices de glyphes et des couleurs, pas la police. Une compatibilité doit donc définir le jeu de caractères associé.
- L'import PNG vise notamment les captures de grilles : les caractères ne sont pas reconnus, et une conversion pixel vers fond de cellule est possible. Ce n'est pas un moteur général de conversion photographique en ASCII.
- Plusieurs échanges passent par la ligne de commande ou de la configuration.
- Le manuel documente des exceptions à l'annulation, notamment la fusion de calques et les modifications de palettes.

Source : [manuel officiel, sections palettes/calques et annexes B, F et G](https://www.gridsagegames.com/rexpaint/manual.txt).

### Lecture visuelle et UX

Les démonstrations montrent une interface compacte : colonne d'outils à gauche, grande zone de dessin à droite, typographie bitmap, cadres fins, peu de décoration. La palette de glyphes reste immédiatement accessible. Certains boutons et états sont très abrégés.

Notre interprétation : conserver la sensation d'atelier dense et précis, tout en donnant plus de lisibilité aux outils, aux états actifs et aux raccourcis. Les démonstrations peuvent illustrer d'anciennes versions ; elles servent ici de références visuelles, pas de preuve d'un test de la version 1.70.

Références inspectées : [thèmes](https://www.gridsagegames.com/rexpaint/features/rexpaint_skins.gif), [sélecteur de couleur](https://www.gridsagegames.com/rexpaint/features/rexpaint_colorpicker_h.png), [calques](https://www.gridsagegames.com/rexpaint/features/rexpaint_layers.gif).

## 2. Décisions confirmées

| Sujet | Décision utilisateur |
| --- | --- |
| Application | Fenêtre native macOS/Linux, développée en Rust. |
| Usages | Illustrations, sprites et cartes, sans spécialisation exclusive. |
| Référence fonctionnelle | REXPaint. Aucune reprise des choix d'un autre projet. |
| Interaction | Souris et clavier ont la même importance. |
| Fonction complémentaire prioritaire | Image de référence sous le dessin. |
| Sorties | Texte à copier et image PNG. |
| Direction artistique | Serial Experiments Lain comme référence principale ; chrome/Y2K, rétro, terminal et touches d'interface Evangelion. |
| Première ouverture | Choix rapide du format avant d'entrer dans l'éditeur. |

Les décisions suivantes sont des propositions cohérentes avec ces réponses, pas des demandes supplémentaires attribuées à l'utilisateur.

## 3. Périmètre V1 proposé

Un atelier local à écran principal unique. Un même document sert à dessiner une illustration, un sprite ou une carte ; les préréglages changent ses dimensions, pas les outils disponibles.

### Socle

- Grille de caractères aux dimensions fixes, zoom et déplacement de la vue.
- Répertoire de départ inspiré du CP437 de REXPaint : lettres, chiffres, ponctuation, symboles, cadres simples/doubles, blocs et ombrages.
- Palette de glyphes organisée en grille ; onglet complet et catégories, derniers glyphes utilisés, aperçu du pinceau.
- Premier plan et fond en couleur, modifiables séparément du caractère ; sélecteur et palette compacte.
- Crayon, gomme, pipette, ligne, rectangle vide/plein, remplissage et texte.
- Sélection rectangulaire, déplacer, couper/copier/coller, annuler/rétablir par geste.
- Image de référence : importer, afficher/masquer, déplacer, redimensionner en conservant ses proportions, régler l'opacité et verrouiller.
- Projet éditable, récupération du brouillon, texte à copier et export PNG.

Le manuel de REXPaint décrit une police de départ de 256 glyphes CP437, des polices bitmap et des correspondances Unicode pour le texte. Nous reprenons ce principe de répertoire graphique déterminé, avec nos propres ressources distribuables. [Manuel, Fonts](https://www.gridsagegames.com/rexpaint/manual.txt)

### Simplifications proposées

- Un plan de dessin et une référence indépendante pour la V1. Des calques de dessin supplémentaires restent un arbitrage, pas une décision utilisateur.
- Pas de conversion automatique de photo : la référence sert à dessiner manuellement par-dessus.
- Sprites statiques ; « carte » désigne une composition de caractères, pas encore un éditeur de niveaux avec collisions ou logique de jeu.
- L'export texte produit de l'UTF-8 ; une correspondance Unicode explicite convertit les glyphes du répertoire. Il ne s'agit pas d'un fichier de commandes ANSI colorées.
- Compatibilité `.xp`, polices arbitraires, export ASCII strict, animation et formats spécialisés hors périmètre proposé pour le premier lot.
- Le raccord automatique des cadres est intéressant pour les cartes, mais sa priorité reste à discuter après le socle.

### Contrat de caractères

Le document conserve un identifiant de répertoire/version et les indices des glyphes. Le rendu bitmap et la table Unicode font partie des ressources de l'application. Chaque glyphe dessinable en V1 doit avoir une sortie texte définie, y compris les symboles graphiques aux emplacements historiquement utilisés par des codes de contrôle. Aucun octet de contrôle CP437 ne doit être émis dans le presse-papiers.

Une case occupe toujours une cellule du document. Saisie et collage doivent signaler les caractères extérieurs au répertoire avant validation ; le texte ne se décale pas silencieusement. Le périmètre ne prétend pas couvrir tout Unicode.

Le rendu texte dépendra de la police de destination. Des glyphes carrés dans l'application peuvent apparaître rectangulaires ailleurs. Le PNG est la sortie fidèle à l'apparence ; la copie texte conserve les caractères et la structure, sans couleurs ni image de référence.

## 4. Direction artistique : une vraie interface terminal dans une fenêtre native

Exigence utilisateur précisée : tout l’intérieur de la fenêtre doit avoir la grammaire d’un terminal. Menus textuels, cadres en caractères, commandes entre crochets, coordonnées et curseur de cellule. Les inspirations Lain/Y2K/Evangelion influencent cette interface terminal ; elles ne la remplacent pas par des panneaux graphiques classiques.

### Traduction des références

| Inspiration donnée | Traduction proposée dans l'application |
| --- | --- |
| Lain, référence principale | Fond presque noir légèrement brun/violet, surfaces sombres, silence visuel, espaces de travail isolés et petites indications techniques utiles. |
| Chrome / Y2K | Texte argenté froid, contrastes de gris et compositions rétro ; aucun bouton brillant, dégradé métallique ou biseau graphique. |
| Terminal | Monospace, alignements stricts, coordonnées et raccourcis visibles, cadres anguleux. |
| Evangelion | Orange de signalisation pour l'action active, étiquettes compactes, géométrie nette. Les grands avertissements restent réservés aux vrais états d'alerte. |

Il s'agit d'une interprétation créative des références utilisateur, pas d'une reproduction de leurs interfaces ou de leurs marques.

### Règles visuelles

- Base sombre avec texte ivoire/gris argenté ; accent orange brûlé proposé, violet très discret en soutien.
- Dessin au centre sur un fond neutre, séparé des couleurs de l'interface.
- Titres de section courts, capitales espacées avec parcimonie ; labels d'action lisibles.
- Cadres en traits de caractères, angles droits et zones alignées sur une grille de texte ; commandes telles que [ Nouveau ] ou [ PNG ], avec cibles manipulables.
- Police d'interface monospace lisible, distincte de la police bitmap du dessin.
- Aucun effet métallique, bruit, flou, clignotement ou scanline appliqué à l'œuvre. Le style repose sur la typographie, les cadres, les espacements et les couleurs.
- États sélectionnés indiqués par forme, texte et couleur. Focus clavier visible sur tous les contrôles.
- Pas de fausses métriques, de télémétrie décorative ni de messages d'erreur permanents.

La maquette propose cette direction avec réglage de l’accent et de la densité du texte. Ses glyphes utilisent une police monospace de prévisualisation ; le dessin bitmap définitif sera validé dans le prototype Rust.

## 5. Parcours utilisateur

### A. Démarrer : un choix rapide

Un panneau compact affiche trois formats et une option personnalisée :

- Sprite : 32 × 32 cellules.
- Illustration : 80 × 50 cellules.
- Carte : 100 × 60 cellules.
- Personnalisé : colonnes et lignes.

Ces dimensions sont des valeurs initiales proposées. Le format n'enferme pas le document dans un usage. Un aperçu des proportions se met à jour, les dimensions restent éditables, et Entrée crée le document. Les cellules carrées sont le réglage de départ proposé dans l'esprit de REXPaint ; un aspect rectangulaire pourra être évalué séparément.

« Ouvrir un projet » reste disponible à ce stade. La référence est facultative : la proposer dans l'éditeur évite d'alourdir le démarrage.

### B. Dessiner : un éditeur, trois zones

- À gauche : outils, glyphes et couleurs, toujours accessibles.
- Au centre : grille et prévisualisation du geste ; aucune fenêtre ne recouvre la zone pendant le dessin normal.
- À droite : petit inspecteur de référence, repliable ; en l'absence d'image, action « Ajouter une référence ».
- En haut : document, état d'enregistrement, copier le texte et exporter PNG.
- En bas : outil/mode actif, position, dimensions, raccourcis contextuels et zoom.

Le choix d'un outil ou d'un glyphe ne modifie jamais le document avant le geste de dessin. Un trait continu compte pour une action annulable. Les formes et le collage ont un aperçu annulable avec Échap.

### C. Souris et clavier

Les deux passent par les mêmes commandes du cœur : pas de fonctions réservées à une seule méthode d'entrée.

- Souris : tracer, glisser une sélection, cliquer les glyphes, zoomer au pointeur ; Espace + glisser déplace uniquement la vue.
- Clavier, canevas actif en mode dessin : flèches pour déplacer le curseur, Entrée pour appliquer une cellule ; formes en deux points avec Entrée pour l'origine puis la fin ; Échap annule.
- Sélection au clavier : outil Sélection, Entrée pose l'origine, flèches étendent la zone, Entrée confirme ; une commande Déplacer permet ensuite de repositionner le contenu.
- Palette : navigation par flèches et validation par Entrée ; retour explicite au canevas. Le focus décide si les flèches parcourent les glyphes ou déplacent le curseur.
- Texte : mode visible, saisie en remplacement ; retour à la ligne à la colonne de départ du bloc, pas de reflow du dessin ; Échap sort de la saisie.
- Raccourcis système : Cmd sur macOS, Ctrl sur Linux ; ils ne doivent pas se déclencher pendant une composition de texte.

Les lettres de raccourcis et la règle précise de débordement seront fixées dans le prototype d'interaction. Aucune disposition de clavier n'est présupposée à partir d'un autre projet.

### D. Image de référence

1. Importer une image depuis le sélecteur de fichier ou par glisser-déposer.
2. Ajuster la référence à la grille en conservant ses proportions ; proposer « Contenir » ou « Remplir » et un centrage.
3. Régler sa position, son échelle et son opacité dans le même repère que le dessin.
4. Verrouiller sa transformation, puis dessiner. Masquer/afficher la référence doit être immédiat.
5. Une vue « Tracé » peut atténuer uniquement l'affichage du dessin pour voir la référence derrière les cases à fond opaque. Cette aide visuelle ne modifie ni les cellules ni leur export.

La référence est copiée dans le projet pour rester disponible après déplacement du fichier source. Son affichage suit le zoom et le déplacement de la grille. Elle reste exclue du texte et du PNG du dessin.

### E. Copier / exporter / reprendre

- Copier le texte : sélection active si elle existe, sinon toute la grille ; afficher la portée. Conserver les espaces d'alignement et les retours à la ligne. Un aperçu montre exactement ce qui sera copié.
- PNG : dimensions résultant du nombre de cellules et de la police bitmap ; échelles entières ×1, ×2, ×4 ; fond transparent ou couleur choisie. La référence, la grille, la sélection et les aides de dessin n'apparaissent jamais.
- Enregistrer : conserver le dessin, le répertoire, la palette et l'image de référence avec ses réglages. Exporter ne remplace pas la sauvegarde du projet.
- Reprendre : rouvrir un projet avec le même contenu et la référence alignée ; proposer un brouillon récupéré après interruption.

## 6. Architecture technique proposée

### Cœur et pipeline

**Entrées souris/clavier → commandes → document + historique → composition → rendu de l'éditeur / PNG / texte.**

Le cœur Rust gère la grille, les attributs des cellules, les sélections et les commandes. L'interface ne porte pas les règles de dessin. Un geste conserve un ensemble de changements réversibles ; l'historique est borné par un budget mémoire à fixer, sans promesse d'annulation illimitée.

La prévisualisation reste un état temporaire distinct. La référence est une ressource et une transformation du document, jamais une conversion implicite en cellules.

### Rendu

- Rust + winit + wgpu : stack retenue pour le prototype. winit gère la fenêtre et les entrées ; wgpu le rendu GPU. L’interface terminal est construite sur une grille logique avec ses propres contrôles textuels, cadres et états de focus. Aucun navigateur ni WebView.
- Atlas bitmap pour le dessin, zoom aux facteurs adaptés à la netteté ; dimension des cellules indépendante de la taille de police de l'interface.
- Ordre d'affichage : fond de vue → référence → fond des cellules → glyphes → sélection et curseurs.
- Export PNG : composition depuis le document avec le même atlas, sans référence ni effets d'interface. L'opacité d'aide au tracé ne s'applique pas.
- Une cellule vide et un espace avec un fond opaque restent distincts. La transparence du fond est un état explicite.

Deux espaces de rendu distincts : grille de l’interface à taille lisible, et grille du document avec sa propre caméra de zoom/déplacement. Une table de focus et des actions sémantiques assurent l’usage souris/clavier. Les dialogues de fichiers peuvent rester ceux du système. L’accessibilité des contrôles et la saisie composée seront à traiter explicitement, car un rendu sur mesure ne les fournit pas automatiquement.

Sources : [winit](https://github.com/rust-windowing/winit), [wgpu](https://wgpu.rs/). La stack reste à valider sur les deux systèmes par un prototype exécutable.

### Fichiers et plateformes

Projet local versionné, contenant métadonnées, cellules et ressources embarquées. Le conditionnement exact sera précisé avant l'implémentation. Écriture dans un fichier temporaire, puis remplacement atomique ; brouillon distinct du fichier utilisateur.

Adaptateurs dédiés : rfd pour les dialogues de fichiers, arboard comme candidat pour le presse-papiers, image pour le décodage des références et l’encodage PNG, serde pour les métadonnées versionnées. Ces bibliothèques restent à vérifier et à figer avant implémentation. Adaptateurs également pour les raccourcis et le packaging. Un `.app` pour macOS ; un conditionnement Linux à choisir après définition des distributions et architectures cibles. Tests réels sur macOS et Linux, notamment focus, clavier, presse-papiers et DPI.

## 7. Plan de réalisation des spécifications puis du prototype

1. Valider la direction visuelle avec la maquette et le parcours format → éditeur.
2. Fixer les comportements détaillés : outils, focus, sélection, texte, référence, sauvegarde et exports.
3. Fixer le répertoire CP437 et sa table Unicode, la police bitmap et le modèle documentaire.
4. Réaliser un prototype Rust ciblant rendu net, saisie clavier, dessin souris et alignement de référence sur les deux systèmes.
5. Construire une tranche complète : créer → dessiner → corriger → enregistrer → rouvrir → copier/exporter.
6. Ajouter les outils restants, mesurer la fluidité et terminer le conditionnement.

Cette phase reste une phase de conception ; la création de la maquette n'est pas un démarrage de l'application Rust.

## 8. Critères de recette proposés

- Créer chacun des formats sans parcours d'onboarding supplémentaire.
- Réaliser une petite illustration à la souris et un motif au clavier ; les deux sont rééditables de la même façon.
- Tracer rapidement sans trou entre les événements souris, puis annuler le trait entier.
- Créer et déplacer une sélection au clavier ; annuler rend exactement la grille précédente.
- Zoomer et déplacer la vue sans désaligner la référence ni modifier le document.
- Vérifier qu'une cellule avec fond opaque peut masquer la référence et que l'aide au tracé n'affecte pas le PNG.
- Fermer, déplacer le projet, rouvrir : retrouver dessin, palette et référence sans dépendance au fichier image d'origine.
- Copier les glyphes du répertoire avec la table prévue, sans codes de contrôle ; conserver la structure de la sélection ou de la grille.
- Comparer le PNG au dessin, référence masquée et aides désactivées : glyphes, couleurs, proportions et transparence doivent correspondre.
- Vérifier la netteté et les interactions à plusieurs échelles d'écran, sur macOS et les environnements Linux retenus.

## 9. Arbitrages restant ouverts

- Valider la palette et la densité de l’interface terminal ; le caractère terminal de l’interface est confirmé.
- Confirmer la simplification à un seul plan de dessin pour la V1 ; l'image de référence est déjà retenue.
- Versions minimales de macOS, distributions Linux et architectures CPU à préciser avant packaging.
- Langue de l'interface et nom définitif à confirmer. La maquette utilise des libellés français et « Ditto » comme nom de travail.
