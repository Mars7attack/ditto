# Shaders — Ditto 0.4

La fenêtre **Shaders / F7** ajoute une finition au dessin : lumière, couleurs,
trames et texture. Elle suit le principe des effets empilables présenté par
[Script Slayer](https://studioaaa.com/script-slayer/) et
[Dither Boy](https://studioaaa.com/product/dither-boy/) : combiner des effets,
changer leur ordre, régler leur intensité et retrouver une recette. Les effets
et leur implémentation WGSL sont propres à Ditto ; aucun code ou asset de ces
applications n’est incorporé.

## Utilisation

1. Dessiner ou ouvrir un projet, puis **Shaders F7**.
2. Choisir un look **Néon / CRT / Riso / Irisé**, ou **+** un effet dans le catalogue.
3. Sélectionner sa ligne pour régler le mélange et ses trois paramètres avec **− / +**.
4. Modifier les deux encres du duotone en cliquant sur leurs valeurs hexadécimales.
5. Monter ou descendre un effet avec **↑ / ↓** ; sa case le désactive sans le supprimer.
6. Zoomer dans l’aperçu avec la molette ou le trackpad, puis glisser pour déplacer la vue.
   Les boutons **− / +**, **Ajuster** et **100 %** permettent aussi de cadrer les détails.
7. Comparer le résultat au dessin original avec **Comparer avant/après**.
8. **Exporter PNG** pour produire le rendu, ou fermer pour continuer le dessin.

L’aperçu sur damier montre la transparence. Depuis 0.4.4, sa vue est indépendante
et navigable : même sensibilité de scroll que le canevas, zoom ancré sous le pointeur,
déplacement par glisser avec le bouton gauche, même en dehors de la zone pendant le
geste. Le scroll au-dessus des réglages ne zoome pas. Les raccourcis **+ / −**, **0**
(ajuster), **1** (100 %) et **Alt + flèches** fonctionnent dans le panneau ; les boutons
restent accessibles avec Tab/Entrée. Le pourcentage est relatif aux cellules naturelles
8×16 en points logiques, comme dans le canevas.

Le zoom et le déplacement restent en place lors des réglages, changements de look,
comparaisons, annulations et réouvertures du panneau. **Ajuster** recentre le dessin
entier et suit les redimensionnements de fenêtre. Une nouvelle ouverture de document
réinitialise cette vue. Elle n’affecte ni le cadrage du canevas, ni l’historique du
dessin, ni l’export, et ne provoque pas de nouveau calcul des shaders.

La case globale **Effets** est un
réglage enregistré ; **Comparer avant/après** est une vue temporaire qui n’affecte
jamais l’export. Fermer conserve les réglages. **Vider la pile** et l’application
d’un look sont annulables, comme les autres modifications.

**Sauver look / Charger look** utilise le format JSON validé `.ditto-shaders`.
Un look remplace la pile entière. Les projets et les brouillons contiennent
aussi la pile. Cmd/Ctrl S enregistre le projet depuis la fenêtre Shaders ;
Cmd/Ctrl Z et Shift Z annulent et rétablissent. Les boutons sont exposés à
AccessKit et accessibles par Tab puis Entrée.

## Blur et Blur des contours (0.4.5)

Les deux effets sont accessibles dans **Ajouter un effet**, avec le même réglage
**Mélange** que le reste de la pile. Ils sont annulables, enregistrés dans le projet
et dans les looks, et inclus dans le PNG.

- **Blur** : flou gaussien global. **Rayon / px** règle son étendue ; **Horizontal**
  et **Vertical** règlent la proportion appliquée sur chaque axe. Mettre un axe à zéro
  permet un flou uniquement dans l’autre direction. Rayon nul ou deux axes nuls
  rendent l’effet neutre.
- **Blur des contours** : adoucit les transitions de couleur et de transparence en
  conservant les régions dont le contraste local est faible. **Rayon / px** règle
  l’étendue, **Seuil** sélectionne les transitions à traiter (plus haut = moins de
  contours), **Douceur** rend cette sélection progressive. Il fonctionne aussi sur
  fond opaque. Les traits très fins peuvent être floutés presque entièrement, car
  ils contiennent surtout des contours.

Les rayons sont exprimés dans les pixels du document : leur taille reste cohérente
au zoom et aux exports ×1/×2/×4. Le flou diffuse la couleur et la transparence sans
ajouter de lumière, contrairement au Glow. L’ordre de la pile reste significatif.

## Pipeline

Le document garde ses cellules et une recette indépendante, limitée à huit effets.
Une modification de paramètres n’altère ni les glyphes, ni leurs couleurs, ni la
référence. L’historique mémorise seulement les recettes pour ces changements,
sans recopier la grille ou l’image de référence.

Le rendu des cellules fournit une image RGBA transparente à la résolution de l’atlas
SF Mono (×4, soit 32×64 par cellule), avec ses couvertures antialiasées complètes.
Depuis 0.4.3, activer les effets ne réduit plus systématiquement les caractères à
8×16 avant de les agrandir. Cette image est conservée en cache tant que les cellules,
les dimensions et la résolution restent identiques ; changer le zoom ne la reconstruit
pas. Les rayons et trames reçoivent l’échelle effective pour garder leur taille dans
le document. Le filtrage final interpole les couleurs en alpha prémultiplié afin
que les pixels transparents ne créent pas de bord sombre.

Les passes de shaders s’exécutent sur le GPU dans l’ordre de la pile ; trois textures
servent de surfaces intermédiaires. Le glow utilise un seuil de luminance, un flou horizontal, un flou vertical et
une composition lumineuse. Le flou échantillonne chaque texel dans son rayon,
pour éviter des halos quadrillés autour des traits fins en haute résolution. Les mélanges tiennent compte de l’alpha prémultiplié
pour conserver la couleur des halos transparents. Blur utilise deux passes gaussiennes séparables en alpha prémultiplié, puis une
composition avec le mélange. Blur des contours réutilise ce flou et construit un
masque progressif à partir du contraste de son voisinage lissé ; cela évite les
anneaux nets autour des bords antialiasés. Les autres effets prennent une passe chacun. Le grain utilise un hash entier déterministe.

L’export PNG rasterise à l’échelle demandée puis exécute exactement le même WGSL
sur un contexte wgpu hors écran. Les rayons et les trames sont exprimés en pixels
du document et adaptés aux exports ×1, ×2 et ×4. Le fond opaque, s’il est demandé,
est ajouté après les effets. L’export tourne dans la tâche de fond existante ; il
ne capture pas la fenêtre. La référence et les guides d’édition sont exclus.

Le canevas normal affiche le résultat avec shaders ; les curseurs et guides sont
ajoutés ensuite. Les aperçus provisoires des formes, sélections déplacées et blocs
collés restent des aides d’édition ; les shaders sont recalculés sur leur résultat
une fois le geste confirmé. Aucun timer d’animation n’est ajouté.

## Format et limites

Les nouveaux projets sont en **V5**, avec un champ `shaders` dans le manifeste
et une entrée séparée pour les guides. Les V4 restent lisibles avec leurs shaders.
Les projets V1, V2 et V3 restent lisibles, avec une pile vide. Le profil de glyphes
reste `ditto-sfmono-v3`. Les versions antérieures à 0.5 ne peuvent pas ouvrir les V5.
Les projets et looks contenant Blur ou Blur des contours nécessitent Ditto 0.4.5
ou ultérieur ; les lecteurs plus anciens rejettent ces effets inconnus. Les anciennes
recettes restent compatibles dans la nouvelle version.
Les recettes invalides, les valeurs non finies ou hors limites, les types inconnus,
les piles de plus de huit effets et les fichiers de look dépassant 64 Kio sont rejetés.

Les effets restent dans les limites de la grille ; laisser une marge pour les
halos. Leurs dimensions doivent aussi être prises en charge par le GPU. Une erreur
de rendu est signalée ; un export en échec ne remplace pas le PNG précédent.
L’aperçu choisit ×4, puis ×2 ou ×1 si les dimensions du GPU ou un budget de
16 millions de texels imposent une réduction. Une grande grille déjà prise en
charge en ×1 reste acceptée jusqu’à la limite précédente de 64 millions de texels.
Ces très grandes grilles peuvent donc conserver un aperçu moins détaillé au zoom.
La taille maximale de grille et la limite de 64 millions de pixels à l’export
restent les mêmes. Le profilage prolongé sur les très grandes grilles reste à faire.

Cette version propose des effets fixes et paramétriques. L’animation, la timeline
et l’import de programmes de shaders personnalisés restent hors périmètre.

## Validation reproductible

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo test --locked --test shaders -- --include-ignored
cargo run --locked --example shader_proof
cargo run --locked --example blur_proof
cargo run --locked -- --smoke-dir validation/runtime/shaders/smoke
cargo run --locked -- --shader-quality-smoke-dir validation/runtime/shader-sharpness/native
```

Le test GPU est ignoré par défaut sur les machines sans adaptateur graphique et
doit être exécuté explicitement lors de la validation native. Il vérifie que
chaque effet change les pixels, que les zones vides restent transparentes, que
le glow déborde des glyphes, que l’ordre change le rendu, que le grain est stable
et que le PNG utilise le même traitement. L’exemple produit un projet de démo
et les rendus des quatre looks. Le smoke ouvre une fenêtre native, manipule la
pile, enregistre, recharge et exporte, puis capture le framebuffer réellement
présenté dans `shaders-window.png`, puis teste zoom/déplacement/comparaison et capture
`shaders-zoomed-window.png` avant de quitter.


Le scénario de netteté capture le rendu direct, un shader neutre actif, puis un look
Néon, à 64 points de hauteur par cellule. Il compare les pixels à l’intérieur des
cellules (tolérance 3/255). Le bord d’un texel est traité séparément : une image
aplatie filtre entre cellules voisines, tandis que les quads de glyphes directs sont
coupés à leur frontière. Le rapport conserve aussi les écarts sur ces frontières.


`blur_proof` produit une comparaison Original / Blur / Blur des contours, avec des
lettres, un aplat et une texture de faible contraste. Les tests GPU vérifient aussi
les axes indépendants, l’absence d’assombrissement des couleurs aux bords transparents,
les effets neutres, la conservation des détails peu contrastés, les contours opaques,
la taille des effets aux différentes échelles et les exports d’une pile mixte.
