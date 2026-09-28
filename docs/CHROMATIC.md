# Chromatique : modèle et transparence

L’aberration chromatique vient de la dépendance de l’indice de réfraction à la
longueur d’onde. Elle décale les composantes de la lumière : elle n’invente pas
du rouge dans une source uniquement bleue. Référence :
[PBRT, indice de réfraction et dispersion](https://www.pbr-book.org/4ed/Reflection_Models/Specular_Reflection_and_Transmission).

Ditto utilise une approximation artistique RGB directionnelle, avec trois
positions d’échantillonnage, et non une simulation spectrale complète de lentille.
Les paramètres existants restent compatibles : distance en pixels du document,
angle et balance rouge/bleu. La résolution d’aperçu ou d’export multiplie la
distance, ce qui conserve l’échelle de l’effet.

## Traitement

1. Décoder sRGB en lumière linéaire avant toute interpolation.
2. Pour chaque texel et canal `k`, calculer la lumière prémultipliée
   `P[k] = linearRGB[k] * alpha`.
3. Construire une couverture relative `A[k] = alpha * linearRGB[k] / max(linearRGB)`.
   Une couleur absente contribue donc zéro à l’opacité. Les divisions sont
   protégées et les pixels noirs opaques restent ancrés à leur position originale.
4. Échantillonner bilinéairement lumière et couverture aux positions rouge et
   bleue décalées ; conserver le vert au centre. Aucun arrondi au pixel entier.
5. Assembler les trois lumières et leur enveloppe d’opacité `max(A.r, A.g, A.b)`.
   Mélanger avec l’original en lumière linéaire prémultipliée, puis déprémultiplier
   et encoder en sRGB pour le reste du pipeline.

L’interpolation en lumière linéaire et la prémultiplication suivent les principes
du [W3C, interpolation des couleurs](https://www.w3.org/TR/css-color-4/#interpolation).
Le choix de l’enveloppe de couverture est propre à Ditto : un fichier RGBA
classique n’a qu’un alpha et ne peut pas représenter trois opacités spectrales
indépendantes. Cette reconstruction évite les silhouettes noires qu’ajoutait
l’ancien `max(alpha_source, alpha_rouge, alpha_bleu)` même si le canal était nul.

Un bleu pur sombre conserve son bleu et sa luminosité ; il se décale sans fantôme
rouge/noir. Le blanc se sépare en franges RGB. Un réglage de distance zéro est
strictement neutre. Le noir opaque est préservé. Aperçu et PNG utilisent le même WGSL.

## Vérifications

- Régression observée sur l’ancien rendu : pixel noir d’alpha 192 à la place
  d’un canal absent autour d’une primaire sombre.
- Primaires sombres, alpha partiel, RGB cachés sous alpha zéro, noir/gris/blanc,
  interpolation à 0,25 pixel et moyenne linéaire de deux rouges (146 plutôt que 128).
- Distance et conservation de masse aux échelles 1×, 2× et 4×, angles 0/45/90°.
- Captures natives avant/après et PNG de bleus pur, sombre, marine, blanc, cyan et rouge.
