# Changelog

## À paraître

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
