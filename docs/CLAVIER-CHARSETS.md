# Ditto — mode clavier, charsets et glyphes étendus

Depuis la version 0.4.1, les mappings utilisent uniquement **36 sources : A–Z et 0–9**.
Le braille est une sélection de 148 formes : quatre hauteurs et deux pages Espacés.
L’interface, le dessin et les PNG gardent SF Mono et les cellules 8×16.
Les projets V1 à V4 sont lus ; les sauvegardes utilisent V4 (avec la pile de shaders).

## Utilisation

Le bouton `[Mode : souris]` / `[Mode : clavier]` de la barre supérieure bascule
entre la peinture habituelle et la saisie inspirée d’Asciitor. Le raccourci est
Cmd/Ctrl + Maj + M. La bascule ne transforme pas le dessin et conserve le curseur.

En mode clavier :

- Choisir un charset dans le panneau qui remplace la bibliothèque de glyphes.
- La touche source est orange ; son glyphe est affiché à côté en clair.
- Frapper une touche insère son glyphe à la position courante puis avance d’une case.
- Flèches : déplacer le curseur. Maj + flèches : étendre une sélection.
- Cliquer dans le dessin positionne le curseur ; glisser sélectionne une zone.
- Une frappe sur une sélection la remplace à partir de son coin supérieur gauche.
  Le glyphe est toujours inséré ; les réglages FG/BG décident des couleurs appliquées.
- Entrée : début de la ligne suivante. Espace : caractère vide puis avance.
- Tab : insérer des espaces jusqu’au prochain taquet de quatre colonnes ; Maj-Tab recule.
- Retour arrière : reculer et effacer. Suppr : effacer la sélection ou la cellule courante.
- La fin de ligne n’agrandit ni ne décale le dessin ; Entrée permet de continuer. La fin
  du document ne revient pas au début de la dernière ligne.
- Annuler/rétablir restaure aussi le curseur et la sélection des opérations clavier.
- Cliquer une correspondance dans le panneau applique la même insertion qu’une frappe.
- F3 donne accès au choix du charset ; F6 rend le focus au canevas.
- **MacBook : Cmd + ↑ / ↓** passe à la banque précédente/suivante, sans Fn.
  Sous Linux : Ctrl + ↑ / ↓. Les boutons et Page précédente/suivante restent disponibles.
  Le changement boucle de la dernière banque à la première ; le curseur reste en place.

La table entière tient dans le panneau, même en fenêtre compacte. Seules les touches
ayant un glyphe dans la banque courante sont affichées. Une banque change les glyphes
affectés aux mêmes touches, jamais les cellules déjà dessinées.

## Charsets

| Preset | Glyphes | Banques |
| --- | ---: | ---: |
| Contours ASCII | 94 | 3 |
| Texture ASCII | 94 | 3 |
| Traits Unicode | 84 | 3 |
| Blocs et ombrages | 79 | 3 |
| Mosaïques / sextants | 79 | 3 |
| Points / braille | 148 | 4 hauteurs + 2 pages Espacés |

Les correspondances des 26 lettres de la première banque des quatre premiers presets
restent celles d’Asciitor. Les glyphes supplémentaires utilisent les chiffres, puis les
banques suivantes. Les blocs, ombrages et sextants conservent leur catalogue complet.

### Braille : hauteurs et motifs espacés

La hauteur désigne l’étendue entre la première et la dernière rangée de points,
pas le nombre total de points. Les banques par hauteur contiennent les formes
sans rangée vide intermédiaire ; les motifs séparés et les blocs à trous intérieurs
sont regroupés dans Espacés.
Les banques se changent avec Cmd + ↑ / ↓ sur Mac (Ctrl sous Linux), les flèches
du panneau ou Page précédente/suivante.

| Hauteur | Formes | Sélection |
| --- | ---: | --- |
| 1 point | 12 | Point gauche, droit ou paire, aux quatre positions verticales |
| 2 points | 25 | Colonnes et rectangles hauts/médians/bas, diagonales et angles aux trois positions |
| 3 points | 22 | Colonnes et pentes, sans rangée vide intermédiaire |
| 4 points | 19 | Colonnes, diagonales, remplissages et quatre variantes à sept points |
| Espacés / lignes | 26 | Lignes séparées et motifs diagonaux de quatre points sur trois rangées |
| Espacés / blocs | 36 | Groupes séparés, coins opposés et quatre variantes à cinq points |

Dans chaque hauteur, **A = colonne gauche, Z = colonne droite, E = double colonne**,
alignées en bas de la cellule. Chaque forme possède ses symétries horizontale et
verticale dans la même banque, à l’intérieur de sa hauteur et sans déplacer son alignement. Le braille vide est omis : Espace remplit déjà ce rôle.
La bibliothèque souris conserve les 256 motifs pour les besoins ponctuels.

Depuis 0.4.8, la banque **4 points** inclut les quatre formes à sept points :
**H → ⣷, J → ⣾, K → ⡿, L → ⢿**. Une colonne est pleine, l’autre a trois points
contigus, alignés en haut ou en bas.

Depuis 0.4.6, la banque **2 points** comprend les angles alignés en bas sur
**K → ⡤, L → ⢤, M → ⣄, W → ⣠**. Le rectangle bas reste sur **E → ⣤**, avec les
colonnes fines **A → ⡄** et **Z → ⢠**. Les angles correspondants alignés en haut sont
ajoutés sur X/C/V/B. Les 17 correspondances précédentes sont conservées.

Depuis 0.4.7, les 18 motifs espacés ont quitté les pages 3 et 4 points et sont
regroupés dans **Espacés / lignes**. Leurs touches changent avec ce regroupement :

| Motif haut / bas | Hauteur 3 | Hauteur 4 |
| --- | --- | --- |
| 2 / 2 | A → ⣒ | P → ⣉ |
| gauche / gauche | Z → ⡂ | Q → ⡁ |
| droite / droite | E → ⢐ | S → ⢈ |
| gauche / droite | R → ⢂ | D → ⢁ |
| droite / gauche | T → ⡐ | F → ⡈ |
| gauche / 2 | Y → ⣂ | G → ⣁ |
| droite / 2 | U → ⣐ | H → ⣈ |
| 2 / gauche | I → ⡒ | J → ⡉ |
| 2 / droite | O → ⢒ | K → ⢉ |

Depuis 0.4.10, **L → ⣢** correspond aux positions **3, 6, 7, 8**, selon la
numérotation ligne par ligne donnée par l’utilisateur. Les variantes sont **M → ⣔**,
**W → ⡲**, **X → ⢖**, puis les versions alignées en haut **C → ⠵**, **V → ⠮**,
**B → ⠝**, **N → ⠫**. Les 18 mappings précédents restent en place.

```text
Positions     L / 3678
1 2           · ·
3 4           ● ·
5 6           · ●
7 8           ● ●
```

Depuis 0.4.11, le motif **3567 → ⡦** est disponible sur **0** dans
**Espacés / lignes**. Son miroir horizontal est sur **1 → ⢴** (4568),
et les versions alignées en haut sont sur **2 → ⠗** (1345) et **3 → ⠺** (2456).
Ce motif est symétrique verticalement dans sa hauteur de trois rangées ;
ces quatre formes couvrent donc ses orientations et ses deux alignements.
La page contient 30 formes, avec les touches précédentes conservées.

```text
Positions     0 / 3567
1 2           · ·
3 4           ● ·
5 6           ● ●
7 8           ● ·
```

Depuis 0.4.12, **4 → ⡢** correspond au motif **367**, et **5 → ⢔** à son
miroir **458**. Les versions alignées en haut sont **6 → ⠕** (145) et
**7 → ⠪** (236). La réflexion verticale dans ces trois rangées donne le même
motif. Espacés / lignes contient maintenant 34 formes ; les anciens mappings
restent en place.

```text
Positions     4 / 367
1 2           · ·
3 4           ● ·
5 6           · ●
7 8           ● ·
```

La page suivante, **Espacés / blocs**, ajoute les groupes demandés : un carré de
quatre points ou l’un des quatre angles de trois points, une rangée vide, puis
un point gauche/droit ou une ligne de deux points. Chaque motif est suivi de sa
version inversée haut/bas. Par exemple : **A → ⣛** (4 en haut, 2 en bas),
**Z → ⣭** (2 en haut, 4 en bas), **E → ⡛** (4 en haut, 1 en bas), **R → ⣥**
(1 en haut, 4 en bas).

Depuis 0.4.8, deux motifs à six points complètent cette page : **4 → ⣫** et
**5 → ⣝**, soit deux angles de trois points opposés. Les trous sont en diagonale
au centre (points pleins `●`, trous `·`) :

```text
4 / ⣫     5 / ⣝
● ●       ● ●
● ·       · ●
· ●       ● ·
● ●       ● ●
```

Depuis 0.4.9, **6 → ⢫** reprend le motif de la touche 4 (⣫) en retirant uniquement
le point **en bas à gauche**. Ses symétries sont sur **7 → ⡝, 8 → ⣜, 9 → ⣣**.
La page Espacés / blocs utilise désormais les 36 touches directes.

```text
4 / original   6 / modifié
● ●            ● ●
● ·            ● ·
· ●            · ●
● ●            · ●
```

Les 62 formes espacées ou à trous restent dans le même charset Points / braille, sur deux
pages voisines de 26 et 36 touches, afin de respecter la limite de 36 touches directes.
Cmd + ↑ / ↓ passe entre les pages. Les banques de hauteur retrouvent leurs anciens
mappings de formes continues ; les glyphes enregistrés dans les dessins ne changent pas.

### Touches directes sur Mac, AZERTY et QWERTY

Les lettres suivent la disposition active, avec majuscules équivalentes. La rangée
physique 0–9 active les chiffres **sans Maj**, même quand elle produit `&`, `é`, `à`, etc.
sur AZERTY. Un pavé numérique produisant des chiffres fonctionne également.
Aucune ponctuation ni aucun caractère Option/AltGr n’est une source de mapping.

Ce routage physique est limité au canevas en mode clavier. Les champs de dimensions,
couleurs, dialogues et compositions IME gardent leur saisie normale. Cmd/Ctrl restent
prioritaires ; Option/AltGr ne dessinent pas de glyphe mappé. Alt + flèches déplace
la vue. Les compositions et les collages restent littéraux. Les touches libres
d’une banque ne modifient pas le dessin.

Le mode, le charset et la banque sont des choix de session. Ils ne sont pas sauvegardés
comme une transformation du document. Changer de charset n’affecte pas les cellules
existantes ; les glyphes réellement dessinés sont enregistrés.

## Répertoire et rendu

Le répertoire contient 665 identifiants de glyphes. Les 256 premiers gardent exactement
les caractères et bitmaps de la V1. Les ajouts comprennent les blocs et quadrants
U+2580–U+259F, des fractions et textures du bloc Symbols for Legacy Computing,
les 60 sextants, les 256 motifs braille, des traits complémentaires et le signe euro.

La catégorie Blocs en mode souris contient 397 entrées, espace et carré compris :
32 éléments de bloc, 47 fractions/textures supplémentaires, 60 sextants et 256 motifs
braille. Elle est paginée, comme le répertoire complet, sans perte d’accès aux anciens glyphes.

Les nouveaux masques sont dessinés géométriquement dans une grille de 16 × 16 pixels,
sans dépendre d’une police installée. L’atlas GPU et le PNG lisent le même masque. Les
sextants partagent les 16 lignes de pixels en trois bandes ; la résolution impose une
répartition 6/5/5. Le texte copié utilise les vrais caractères Unicode. Leur affichage
dans une autre application dépend de sa police, notamment pour les sextants récents.

Références des formes et plages : [Block Elements](https://www.unicode.org/charts/PDF/U2580.pdf),
[Symbols for Legacy Computing](https://www.unicode.org/charts/PDF/U1FB00.pdf),
[UnicodeData 18.0](https://www.unicode.org/Public/18.0.0/ucd/UnicodeData.txt).
Aucune police n’a été extraite de ces documents ; les nouveaux masques sont procéduraux.

## Historique de compatibilité V1/V2 (livraison 0.2)

Le champ `glyph` utilise désormais un entier 16 bits en mémoire. Les indices existants
0–255 restent identiques. Les indices supplémentaires sont stables et ajoutés à la suite.

| Contenu du dessin enregistré | Version du projet | Profil |
| --- | --- | --- |
| Uniquement les glyphes d’origine | 1 | `ditto-moderndos-square-v1` |
| Au moins un glyphe étendu | 2 | `ditto-blocks-square-v2` |

Ditto 0.2 lit les deux formats. La sauvegarde reste atomique. La V1 n’accepte pas les
projets contenant des glyphes étendus ; elle les rejette explicitement. Les indices
inconnus et les indices étendus introduits frauduleusement dans un projet V1 sont rejetés,
sans remplacement silencieux par un espace ou `?`.

## Validation

Les tests couvrent le mapping Asciitor historique, les 36 sources, les banques complètes,
le routage de la rangée AZERTY sans Maj, les hauteurs/épaisseurs/symétries du braille,
la géométrie des fractions/quadrants/sextants/braille, les bitmaps V1 inchangés,
l’équivalence atlas GPU/masque PNG, les fichiers V1/V2, les frappes et clics équivalents,
la sélection/remplacement/annulation, la composition, les limites de ligne et les
contrôles accessibles en fenêtre compacte. Voir le rapport de validation pour le résultat
réel de la recette native et des builds.
