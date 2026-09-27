# Guides, recoloration et réglages — Ditto 0.5

## Guides de placement

En mode souris, **D / Guides** dessine à main levée sur un calque transparent,
au-dessus du dessin et de ses shaders. Les points sont libres : ils ne s’alignent
pas sur les cases ASCII. Le calque suit le déplacement et le zoom du document.

**Guides F8**, en haut de la fenêtre, ouvre les contrôles :

- Afficher ou masquer le calque ; un calque masqué ne reçoit pas de nouveaux traits.
- Opacité du calque de 5 à 100 %.
- Épaisseur des prochains traits de 0,1 à 8 largeurs de cellule.
- Couleur des prochains traits, saisie en RVB hexadécimal.
- Tracer, gommer des traits entiers au passage du pinceau, ou tout effacer.

Chaque geste est une seule action annulable. Échap ou une perte de focus annule
le geste en cours. Sortir du canevas interrompt le trait ; revenir ne crée pas de
segment traversant le document. Les guides ignorent la sélection de cellules.
Le redimensionnement les masque hors des nouvelles limites sans déplacer leurs
points ; agrandir à nouveau le document les rend visibles.

Les guides sont enregistrés avec le `.ditto` et dans la récupération automatique,
visibilité et opacité comprises. Ils ne figurent **jamais** dans les exports PNG,
l’aperçu des shaders, la copie de cellules ou le texte.

## Recolorer les caractères

En mode souris, **C / Recolorer** applique la couleur **FG** aux glyphes existants.
Choisir une couleur dans la palette ou saisir sa valeur avec le bouton FG, puis
glisser sur le dessin. Les caractères, leur position et leur fond restent inchangés.
Les cellules vides, y compris le braille vide, sont ignorées.

Les boutons − / + sous les outils, ou **[ / ]**, règlent une empreinte de 1 à
17 cellules de diamètre. Le contour indique les cellules couvertes. Le pinceau
respecte la sélection et interpole le passage entre deux positions de souris.
Quitter le canevas interrompt le passage et évite de relier les points au retour.
Un geste correspond à une annulation. Les masques du crayon ne s’appliquent pas
à cet outil : il modifie toujours et uniquement le premier plan.

En mode clavier, C et D conservent leur correspondance de charset.

## Réglages de l’application

Le bouton **Réglages**, ou **Cmd/Ctrl + virgule**, ouvre l’éditeur d’apparence.
Trois thèmes sont fournis : **Ditto**, **Minuit** et **Papier**. Douze couleurs sont
personnalisables : fond général, panneaux, texte principal et secondaire,
bordures, accent/curseur, texte actif, fond de focus, canevas, grille, sélection et
damier de transparence. Cliquer sur une couleur ouvre la saisie hexadécimale.

Les modifications s’affichent immédiatement. **Enregistrer** conserve les
préférences pour les lancements suivants ; **Annuler / Échap** rétablit le thème
précédent. **Défaut** remet les couleurs Ditto dans l’aperçu. Une indication signale
un contraste trop faible. Les couleurs du dessin et le fond opaque des exports
restent indépendants du thème.

Les préférences sont stockées dans `settings.json` sous le dossier de configuration
Ditto propre à l’utilisateur, via `directories::ProjectDirs`. Elles sont lues au
lancement et remplacées par écriture atomique. Un fichier invalide n’est pas écrasé
à la lecture : l’application revient au thème par défaut et indique l’erreur. Un
échec d’écriture conserve l’aperçu ouvert et les préférences précédentes.

## Organisation technique

Le document contient trois familles de données : cellules, pile de shaders et
calque de guides. Les guides sont des polylignes en coordonnées de document, avec
couleur et épaisseur par trait. Le calque et ses traits utilisent un partage par
`Arc` ; dessiner un guide ne copie pas le tableau de cellules. L’historique choisit
une entrée spécifique aux guides, aux shaders ou aux différences de cellules,
et conserve un instantané complet pour les modifications combinées.

Le pipeline de rendu reste : référence → dessin ASCII ou rendu des shaders →
guides → sélection et curseur → interface. Les guides sont transformés en rubans
triangulés, avec bords adoucis, dans le rendu natif existant. Ils utilisent le même
repère et le même découpage que le canevas mais n’entrent jamais dans sa texture
artistique. L’export continue de rasteriser uniquement les cellules, puis applique
les shaders. Le thème fournit des couleurs sémantiques à la construction de l’UI ;
son aperçu est un brouillon indépendant du document et de son historique.

Le format de projet passe en **V5**, avec une entrée `guides.json` séparée du
manifeste et des cellules. Les versions V1 à V4 restent lisibles. Les anciens
lecteurs ne peuvent pas ouvrir les sauvegardes V5. Le profil et les indices de
glyphes ne changent pas. Le chargeur valide les entrées ZIP, les limites, les
coordonnées finies et les paramètres. Limites : 2 048 traits, 65 536 points au total,
8 192 points par trait, entrée de guides limitée à 8 Mio.

## Validation reproductible

```sh
cargo test --locked
cargo test --locked --test shaders -- --include-ignored
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- --smoke-dir validation/runtime/workspace-v050/debug
```

Le smoke exécute les mêmes routes de gestes que la fenêtre, teste recoloration,
annulation/rétablissement, guides, gomme, sauvegarde/réouverture et export propre.
Il capture le framebuffer Metal : calque visible/masqué, panneau Guides, thèmes
Minuit/Papier et canevas clair. Les réglages et la récupération de ce scénario
sont isolés dans son dossier de sortie. Il ne simule pas une souris physique.
