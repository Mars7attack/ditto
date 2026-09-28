# Changelog

## À paraître

- Export braille : mode d’alignement automatique et réversible avec blancs U+2800,
  pour éviter les contours déformés par les largeurs différentes des espaces et
  glyphes de secours. Scripts de comparaison des captures et de rendu CoreText,
  avec test réel des avances de police sur macOS.
- Palette : le picker FG modifie la case sélectionnée et la brosse ensemble,
  avec annulation/rétablissement synchronisés et sélection distincte des doublons.
- Chromatique : interpolation en lumière linéaire, décalages sous-pixel et
  couverture par canal, sans silhouettes noires sur les primaires sombres.
- Réglages des onze effets : sliders glissables, aperçu immédiat, un undo par
  glissement, clavier et valeurs accessibles.
- Export texte repensé : fichier .txt UTF-8 exact, aperçu zoomable, copie
  monospace/HTML avec repli texte, Markdown et blocs Discord sans lignes coupées.
- Mode clavier indépendant de l’outil souris mémorisé, y compris Guides/Gomme
  guides et Texte. Curseur maintenu dans la vue après annulation/rétablissement
  et affiché au-dessus du contour du canevas, même au bord de la vue.
- Grille : lignes alignées sur les pixels physiques, épaisseur d’un pixel quelle
  que soit la densité de l’écran, sans trous au zoom/pan fractionnaire. Régressions
  GPU sur 30 combinaisons d’échelle d’affichage, zoom et déplacement.

- Sélecteur visuel HSV et champ hexadécimal synchronisés pour toutes les couleurs,
  aperçu avant/après, souris, clavier et teinte accessible.
- Curseur clavier rétabli après les outils Guides/Gomme guides et retours des
  panneaux ; restauration du focus après validation au clavier et recentrage
  du curseur lors du passage en mode clavier.
- Suite de contrats et de régressions étendue, parcours natifs avec assertions
  sur les pixels GPU et commande `scripts/check.sh --native` ; exécution CI
  Xvfb/Mesa avec conservation des captures et journaux.

- Points / braille : huit petits L de trois rangées dans la banque « 3 points »,
  couvrant les quatre coins alignés en haut et en bas. Mappings précédents conservés.

## 0.1.0 — X0005 — 2026-09-28

Première version publique de Ditto, atelier ASCII natif pour macOS et Linux.
Les versions 0.2 à 0.5 mentionnées dans les notes historiques sont des étapes
locales antérieures à la numérotation publique. L’identifiant interne X0005 est
indépendant de la version publique 0.1.0 et du format des fichiers de projet.

- Dessin à la souris, édition au clavier et charsets étendus.
- Pinceau pour recolorer les glyphes sans modifier leurs caractères ou leurs fonds.
- Calque de guides à main levée, entre la référence et les caractères par défaut.
  L’option « Devant les caractères » bascule le calque au premier plan.
- Bouton « Recolorer tous les traits » : applique la couleur choisie à tous les
  guides existants en une seule action annulable.
- Pile de shaders GPU, looks et export PNG utilisant le même moteur de rendu.
- Thèmes Ditto, Minuit et Papier, douze couleurs d’interface personnalisables.
- Sauvegarde atomique, récupération automatique, annulation/rétablissement.
- Projets V6, avec lecture des V1 à V5 ; les guides restent exclus des exports.

Distribution macOS Apple Silicon et Linux x86_64. Le bundle macOS est signé ad hoc,
sans notarisation Apple. Le rendu est validé sur Apple M2 / Metal ; la session
interactive Linux reste à valider. Les sources et la CI sont disponibles sur
[GitHub](https://github.com/Mars7attack/ditto).
