# FTR-001 — Fusionner des PDF

## Statut et source de vérité

Cette fiche précise le parcours de fusion confirmé par l'utilisateur final.
En cas de divergence, `specs/SFG.md` prévaut toujours.

## Intention

Permettre à l'utilisateur de créer un unique PDF à partir d'au moins deux PDF,
dans l'ordre qu'il a choisi.

## Préparer les PDF source

L'utilisateur ajoute des PDF soit en les choisissant dans ses dossiers, soit en
les glissant dans l'application. Il peut également ajouter un dossier : seuls
les PDF présents à la racine de ce dossier sont ajoutés, selon l'ordre
alphabétique de leurs noms. Les sous-dossiers ne sont pas parcourus.

Chaque ajout est vérifié immédiatement :

- les fichiers qui ne sont pas des PDF sont ignorés et l'application affiche un
  récapitulatif des fichiers ignorés ;
- pour chaque PDF protégé par mot de passe, illisible ou inaccessible,
  l'application ne demande aucun mot de passe et propose d'annuler toute la
  préparation ou d'ignorer ce PDF ;
- annuler vide toute la préparation en cours ; ignorer conserve les autres PDF
  valides ; un PDF ignoré doit être ajouté manuellement pour être réessayé.

La liste affiche le nom de chaque PDF ; son chemin complet est disponible au
survol. Le même PDF peut y apparaître plusieurs fois. L'utilisateur peut ajouter
ou retirer des PDF, les réordonner par glisser-déposer, et les déplacer vers le
haut ou le bas avec des actions dédiées.

La fusion n'est disponible que lorsqu'au moins deux PDF valides sont présents.
Si ce n'est plus le cas, la liste est conservée, mais l'action de fusion est
désactivée.

## Définir la sortie

L'utilisateur choisit un nom et un dossier local de destination. Il peut choisir
le dossier ou coller son chemin ; ce dossier doit déjà exister. En cas de dossier
inaccessible ou non inscriptible, il peut corriger la destination sans perdre la
liste des PDF.

L'application ajoute `.pdf` au nom si cette extension est absente. Si le nom est
déjà utilisé dans le dossier de destination, elle ajoute un suffixe numérique :
`document.pdf`, `document-1.pdf`, `document-2.pdf`, et ainsi de suite. Elle
refait cette vérification juste avant l'écriture du résultat.

## Confirmer et exécuter

Avant la création, l'application présente un récapitulatif contenant :

- l'ordre et les noms des PDF source ;
- le chemin complet du PDF final.

L'utilisateur confirme explicitement cette opération. Modifier la liste ou la
destination après l'affichage de ce récapitulatif ramène à la préparation et
impose une nouvelle confirmation. Pendant le traitement confirmé, la
préparation n'est plus modifiable.

L'application affiche l'avancement chiffré du traitement et permet son
annulation à tout moment. Si l'utilisateur tente de fermer l'application, elle
lui permet d'annuler la fermeture et de revenir au traitement, ou d'annuler le
traitement puis de fermer l'application.

Les pages du résultat conservent leur format, leur orientation et leur contenu,
y compris les liens, formulaires, commentaires et signets. Si l'application
détecte une limitation qui l'empêche de préserver un élément interactif, elle
en avertit l'utilisateur.

## Issue du traitement

Lors d'une réussite, l'application affiche une notification brève, ouvre
immédiatement le PDF créé, puis revient à un écran vide.

Lors d'une annulation, elle supprime tous les fichiers créés pour l'opération,
affiche un message simple et revient immédiatement à un écran vide.

Lors d'un échec, elle supprime tout fichier de sortie partiel, affiche un
message simple et conserve la liste des PDF et la destination afin que
l'utilisateur puisse corriger puis relancer l'opération.

## Exigences SFG couvertes

- F-01, F-03, F-07 à F-15, F-17, F-23 à F-27, F-29, F-31 à F-33 et F-37 à
  F-39 ;
- U-01 à U-04 ;
- parcours « Fusionner des PDF » et « Gérer les incidents ».
