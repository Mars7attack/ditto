# Export et partage du texte

Le bouton **Texte** prend un instantané de la sélection, ou de toute la grille
en l’absence de sélection. L’aperçu se zoome à la molette ou avec −/+, se déplace
en glissant et se recentre avec **Ajuster**. Aucun aperçu n’est tronqué par un
nombre maximal de caractères : le clipping concerne seulement la fenêtre.

**Enregistrer .txt** écrit cet instantané en UTF-8, sans BOM, avec fins de ligne LF.
Les espaces de début/fin, lignes vides et glyphes Unicode sont conservés exactement.
L’écriture utilise le remplacement atomique existant. Le fichier contient toujours
la totalité de la portée, indépendamment du format de copie ou du bloc affiché.

## Formats de copie

- **Texte / apps** : texte exact, accompagné d’une représentation HTML `<pre>`
  avec police monospace, interligne et espaces préservés. Les caractères HTML
  sont échappés. L’application de destination choisit le format qu’elle accepte ;
  la version texte reste disponible et sert aussi de repli si le HTML échoue.
- **Markdown** : bloc de code sans coloration syntaxique. La clôture est plus
  longue que toute séquence d’accents graves du dessin, sans altérer ses glyphes.
- **Discord** : blocs délimités par trois accents graves, conformément au
  [formatage Discord](https://support.discord.com/hc/en-us/articles/210298617-Markdown-Text-101-Chat-Formatting-Bold-Italic-Underline).
  Chaque bloc reste sous la [limite standard de 2 000 caractères](https://github.com/discord/discord-api-docs/blob/main/developers/resources/message.mdx),
  clôtures comprises. Le comptage UTF-16 est conservateur pour les glyphes hors BMP.
  La découpe se fait entre des lignes entières. Les boutons précédent/suivant
  choisissent le bloc affiché et copié ; copier puis coller chaque bloc dans l’ordre.

La concaténation des corps Discord avec un saut de ligne restitue exactement
l’instantané. Une ligne trop longue ou trois accents graves consécutifs déclenchent
un message explicite proposant le `.txt`, plutôt qu’une réécriture invisible ou une
troncature. Pour un dessin long à partager en un seul objet, joindre directement le `.txt`.

## Limites et tests

Le texte porte les caractères et leur placement, pas les couleurs, shaders ou
guides. Une destination qui impose une police proportionnelle ou ne possède pas
certains glyphes peut encore modifier l’apparence ; utiliser le bloc de code ou
une police monospace. Le PNG préserve le rendu visuel exact.

Tests : roundtrip UTF-8, sélection, espaces, lignes vides, HTML échappé, clôtures
Markdown, budget Discord aux frontières, Unicode BMP/hors BMP, assemblage sans
perte, navigation des blocs, zoom/pan sans mutation du dessin, disposition minimum
et captures natives. Les tests n’envoient aucun message sur Discord et ne remplacent
pas le presse-papiers de la session utilisateur ; ils vérifient les payloads produits
par le chemin de copie et les fichiers écrits par le chemin d’export.
