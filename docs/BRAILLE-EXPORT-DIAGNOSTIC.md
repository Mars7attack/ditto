# Diagnostic : contours déformés après export du braille

## Mesures sur les deux captures du 28 septembre 2026

Les deux images contiennent **2 069 points**, répartis sur **85 rangées**. Le
nombre de points est identique rangée par rangée. Dans Ditto, leurs abscisses
occupent 91 positions régulières ; dans le résultat externe, elles occupent 322
positions différentes. Le problème est donc une déformation horizontale, et non
une simple différence de taille ou de forme des points.

L’ajustement géométrique des 2 069 centres utilise ce modèle :

```text
x = origine + colonne * avance_espace
    + nombre_de_cellules_braille_avant * (avance_braille - avance_espace)
    + colonne_du_point_dans_la_cellule * espacement_des_points
y = origine + ligne_de_cellules * interligne + sous_ligne_du_point * espacement_vertical
```

| Mesure dans la capture externe | Résultat |
|---|---:|
| Avance des espaces | 13,244 px |
| Avance des cellules braille | 15,040 px |
| Écart cumulé à chaque changement espace/braille | 1,796 px |
| Interligne | 26,000 px |
| Erreur moyenne quadratique du modèle | 0,120 px |

Le résultat reste stable avec des seuils de segmentation de 25 %, 35 % et 50 %.
Une reconstitution diagnostique du texte depuis la grille de points est rendue
avec CoreText, indépendamment du moteur GPU de Ditto. CoreText indique :

- espace U+0020 : **Menlo-Regular**, avance **13,245117 px** ;
- motifs braille : **AppleBraille**, avance **15,039062 px** ;
- motif vide U+2800 : **AppleBraille**, avance **15,039062 px**.

La [substitution automatique de polices de CoreText](https://developer.apple.com/documentation/coretext/)
explique le mélange malgré le choix d’une police monospace. Le test reproduit les
métriques observées ; il ne lit pas les préférences de l’application destinataire.
La taille 22 du script produit ces mesures dans sa bitmap, sans présumer de la
taille en points choisie dans l’interface de l’utilisateur ni de son facteur Retina.

## Correction et limites

L’export détecte les dessins entièrement en braille et remplace les cellules
vides par U+2800, avec une case explicite pour revenir au texte source. Il ne
modifie pas le document ni les caractères portant les points. Les sorties TXT,
texte, HTML, Markdown et Discord passent toutes par la même préparation.

Sur la reconstitution de l’étoile rendue par CoreText après correction, on retrouve
91 colonnes et 85 rangées : **aucune différence dans l’occupation des 7 735
positions de grille**, intersection/union des points = **1,0**. La déformation
cumulative disparaît. Il reste les différences intrinsèques de dessin des points,
d’espacement interne au glyphe et d’interligne entre les polices. Le PNG reste la
sortie destinée à reproduire exactement le raster de Ditto.

Les captures ne permettent pas de distinguer les différents caractères invisibles
de l’original, ni de récupérer ses marges vides complètes. La reconstitution est
donc un support de diagnostic, pas une récupération exacte du fichier utilisateur.
Les tests de production utilisent une fixture synthétique indépendante.

## Reproduire avec les scripts

```sh
uv run scripts/compare-braille-captures.py export.png ditto.png \
  --crop-b 40,315,1980,805 --out validation/runtime/braille-export/comparison

swift scripts/probe-text-layout.swift dessin.txt \
  validation/runtime/braille-export/coretext Menlo 22
```

Le premier script détecte les composants correspondant aux points, compare leur
répartition et estime les métriques. `--crop-a`/`--crop-b` sont `x,y,largeur,hauteur` ;
la seconde image doit présenter une grille de points uniforme. `--threshold`
contrôle la fraction de contraste retenue. JSON, masques et grilles normalisées
sont écrits dans le dossier de sortie, sans toucher aux images d’entrée. Si le
modèle est suffisamment précis, il écrit aussi deux reconstitutions diagnostiques.
Ses dépendances Python sont isolées par `uv` et leurs versions sont fixées.

Le second script, propre à macOS, produit un PNG et un JSON avec police réellement
utilisée, position et avance de chaque glyphe. Il permet de constater la substitution
de police, puis de contrôler l’alignement du fichier corrigé avec le vrai moteur Apple.

Les preuves de cette investigation sont conservées localement sous
`validation/runtime/braille-export/`. Les captures et le dessin de l’utilisateur
ne sont pas ajoutés au dépôt public.
