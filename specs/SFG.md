# Spécification fonctionnelle générale — PDFForge

## Statut du document

Spécification fonctionnelle générale finalisée le 1er septembre 2026. Elle ne
contient que les besoins explicitement confirmés par l'utilisateur final. Toute
nouvelle règle doit être validée avant d'être ajoutée.

## Vision du produit

PDFForge est une application personnelle permettant de manipuler des fichiers
PDF directement sur l'ordinateur de son utilisateur.

## Objectif

L'application doit permettre à son unique utilisateur de fusionner ou de
scinder des fichiers PDF de manière simple et rapide.

## Exigences fonctionnelles confirmées

| Référence | Besoin confirmé                                                                                                                                                                                                           |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| F-01      | L'utilisateur doit pouvoir fusionner des fichiers PDF.                                                                                                                                                                    |
| F-02      | L'utilisateur doit pouvoir scinder un fichier PDF.                                                                                                                                                                        |
| F-03      | Les traitements et les documents doivent rester exclusivement en local sur l'ordinateur. Aucun document ne doit être déposé sur Internet.                                                                                 |
| F-04      | L'utilisateur doit pouvoir créer un PDF distinct pour chaque page d'un PDF.                                                                                                                                               |
| F-05      | L'utilisateur doit pouvoir extraire des pages précises d'un PDF pour créer un nouveau PDF.                                                                                                                                |
| F-06      | L'utilisateur doit pouvoir définir des groupes de pages afin de créer plusieurs PDF distincts.                                                                                                                            |
| F-07      | Les fichiers PDF d'origine doivent rester inchangés après tout traitement.                                                                                                                                                |
| F-08      | L'utilisateur doit pouvoir enregistrer localement chaque PDF obtenu sous un nom personnalisé.                                                                                                                             |
| F-09      | L'utilisateur doit pouvoir organiser librement l'ordre des PDF à fusionner par glisser-déposer.                                                                                                                           |
| F-10      | L'utilisateur doit pouvoir choisir l'emplacement local de sauvegarde lors de chaque opération.                                                                                                                            |
| F-11      | L'application ne doit pas demander le mot de passe d'un PDF protégé.                                                                                                                                                      |
| F-12      | Si le nom choisi existe déjà dans le dossier de sauvegarde, l'application doit ajouter un numéro à la fin du nom du nouveau fichier.                                                                                      |
| F-13      | L'utilisateur doit pouvoir indiquer l'emplacement local de sauvegarde en collant le chemin d'un dossier.                                                                                                                  |
| F-14      | Lorsqu'un PDF protégé par mot de passe ou non lisible est rencontré, l'application doit demander à l'utilisateur s'il souhaite arrêter l'opération ou ignorer ce PDF et poursuivre avec les autres fichiers sélectionnés. |
| F-15      | L'utilisateur doit pouvoir ajouter les PDF à traiter en les choisissant dans ses dossiers ou en les glissant dans l'application.                                                                                          |
| F-16      | L'application doit afficher des miniatures des pages afin que l'utilisateur puisse choisir les pages à extraire ou à grouper.                                                                                             |
| F-17      | Lorsqu'un traitement produit un seul PDF, l'application doit ouvrir ce document obtenu.                                                                                                                                   |
| F-18      | L'utilisateur doit pouvoir masquer individuellement des lignes dans un PDF.                                                                                                                                               |
| F-19      | Dans le nouveau PDF obtenu après un masquage, les informations masquées ne doivent plus pouvoir être récupérées.                                                                                                          |
| F-20      | Lorsqu'une scission produit plusieurs PDF, l'application doit ouvrir uniquement le dossier qui les contient.                                                                                                              |
| F-21      | Pour le masquage, l'utilisateur doit pouvoir sélectionner un mot à la fois ou plusieurs mots en faisant glisser la souris.                                                                                                |
| F-22      | Si le texte d'un PDF ne peut pas être sélectionné, l'utilisateur doit pouvoir dessiner un rectangle pour masquer définitivement une zone de la page.                                                                      |
| F-23      | Avant la création d'un nouveau PDF, l'application doit afficher un récapitulatif et demander la confirmation de l'utilisateur ; elle doit ensuite afficher un message simple de réussite ou d'échec.                      |
| F-24      | L'application doit fonctionner sans connexion Internet.                                                                                                                                                                   |
| F-25      | L'application doit permettre de traiter aussi bien de petits PDF que des documents de plusieurs centaines de pages.                                                                                                       |
| F-26      | L'application ne doit conserver aucun historique des PDF traités après sa fermeture.                                                                                                                                      |
| F-27      | L'application doit afficher l'avancement d'un traitement et permettre à l'utilisateur de l'annuler.                                                                                                                       |
| F-28      | Lorsqu'une scission produit plusieurs PDF, l'utilisateur doit choisir un nom de base une seule fois et l'application doit ajouter des numéros aux documents créés.                                                        |
| F-29      | Lorsqu'un traitement est annulé, l'application doit supprimer tous les PDF qu'elle a créés pour cette opération.                                                                                                          |
| F-30      | Avant de confirmer un masquage, l'utilisateur doit voir un aperçu des zones masquées et pouvoir modifier ou retirer ses sélections.                                                                                       |
| F-31      | Lors d'une fusion ou d'une scission, les pages obtenues doivent conserver leur format et leur orientation d'origine.                                                                                                      |
| F-32      | Lors d'une fusion ou d'une scission, le contenu des PDF doit être conservé, y compris leurs liens, formulaires, commentaires et signets.                                                                                  |
| F-33      | Avant de confirmer une fusion, l'utilisateur doit pouvoir ajouter ou retirer des PDF de la liste.                                                                                                                         |
| F-34      | Pendant un masquage, l'utilisateur doit pouvoir afficher une page en grand et zoomer.                                                                                                                                     |
| F-35      | Les zones masquées doivent apparaître en noir dans le PDF créé.                                                                                                                                                           |
| F-36      | L'utilisateur doit pouvoir préparer plusieurs zones à masquer dans un même PDF avant sa création.                                                                                                                         |
| F-37      | Si l'utilisateur tente de fermer l'application pendant un traitement, l'application doit afficher un avertissement lui permettant d'annuler la fermeture et de revenir au traitement.                                     |
| F-38      | Après un traitement réussi ou annulé, l'application doit revenir à un écran vide.                                                                                                                                         |
| F-39      | L'application doit démarrer dans la langue du système.                                                                                                                                                                    |

## Exigences d'usage confirmées

| Référence | Attente confirmée                                                                                                  |
| --------- | ------------------------------------------------------------------------------------------------------------------ |
| U-01      | L'application doit être simple à utiliser.                                                                         |
| U-02      | L'application doit être rapide.                                                                                    |
| U-03      | L'application est conçue pour un seul utilisateur : son propriétaire.                                              |
| U-04      | Aucun temps maximal de traitement n'est défini à ce stade, y compris pour les PDF de plusieurs centaines de pages. |

## Parcours fonctionnels confirmés

### Fusionner des PDF

L'utilisateur ajoute des PDF depuis ses dossiers ou par glisser-déposer. Il les
organise par glisser-déposer dans l'ordre souhaité. Il choisit un nom et un
emplacement local pour le document obtenu, puis confirme l'opération. Le PDF
créé s'ouvre à la fin du traitement. Les documents d'origine restent intacts.
Les pages conservent leur format, leur orientation et leur contenu, y compris
leurs éléments interactifs.

Une fusion requiert au moins deux PDF. L'utilisateur peut ajouter le même PDF
plusieurs fois. Il peut ajouter des fichiers ou des dossiers : pour un dossier,
l'application ajoute uniquement les PDF situés à sa racine, dans l'ordre
alphabétique, puis l'utilisateur peut modifier librement cet ordre. La liste
affiche le nom de chaque PDF et son chemin complet au survol.

L'application vérifie chaque PDF dès son ajout. Elle ignore les fichiers qui
ne sont pas des PDF et affiche un récapitulatif des fichiers ainsi ignorés. Si
un PDF est protégé par mot de passe, illisible ou inaccessible, elle ne demande
pas de mot de passe et laisse à l'utilisateur le choix d'annuler toute la
préparation ou d'ignorer ce seul PDF. Cette règle s'applique également aux PDF
trouvés dans un dossier. L'utilisateur qui a ignoré un PDF doit le réajouter
manuellement s'il souhaite le traiter ultérieurement. Si moins de deux PDF
valides restent dans la liste, la fusion est désactivée tout en conservant les
PDF valides déjà ajoutés.

Le nom de sortie reçoit automatiquement l'extension `.pdf` si elle est absente.
Le dossier de destination peut être choisi ou renseigné par collage de son
chemin ; il doit déjà exister. L'utilisateur peut corriger un dossier
inaccessible ou non inscriptible sans perdre sa préparation. Si un fichier de
sortie du même nom existe, l'application crée `document-1.pdf`, puis augmente
ce numéro autant que nécessaire. Elle vérifie à nouveau la disponibilité du nom
juste avant l'écriture.

Avant la création, le récapitulatif présente l'ordre et les noms des PDF source
ainsi que le chemin complet du PDF final, puis demande confirmation. Toute
modification de la préparation impose une nouvelle confirmation. Après la
confirmation, la préparation ne peut plus être modifiée pendant le traitement.
Si l'application détecte qu'elle ne peut pas préserver un élément interactif,
elle en avertit l'utilisateur.

Le traitement affiche un avancement chiffré et peut être annulé à tout moment.
Une annulation ou une erreur supprime tout fichier de sortie partiel créé. Après
une annulation, l'application revient immédiatement à un écran vide. Après une
réussite, elle affiche une notification brève, ouvre immédiatement le PDF créé,
puis revient à un écran vide. En cas d'échec, elle affiche un message simple et
conserve la préparation pour permettre une correction et une nouvelle tentative.

### Scinder un PDF

L'utilisateur ajoute un PDF et visualise les miniatures de ses pages. Il peut
créer un PDF par page, extraire des pages précises ou définir des groupes de
pages afin de produire plusieurs PDF. Il choisit leurs noms et leur emplacement
local, puis confirme l'opération. Lorsque plusieurs PDF sont créés, seul leur
dossier est ouvert. Le document d'origine reste intact.
Les pages conservent leur format, leur orientation et leur contenu, y compris
leurs éléments interactifs.

### Masquer définitivement des informations

L'utilisateur ajoute un PDF. Il sélectionne les mots à masquer un à un ou par
glisser-déposer. Si le texte ne peut pas être sélectionné, il dessine un
rectangle sur la zone à masquer. Il peut afficher la page en grand, zoomer et
préparer plusieurs zones noires à masquer. Après confirmation, l'application crée un
nouveau PDF sans les informations masquées, qui ne doivent pas pouvoir être
récupérées. Avant confirmation, il peut vérifier, modifier ou retirer les
zones sélectionnées. Le document d'origine reste intact.

### Gérer les incidents

Si un PDF protégé par mot de passe ou illisible est rencontré, l'application ne
demande pas de mot de passe. Elle laisse à l'utilisateur le choix d'arrêter
l'opération ou d'ignorer ce PDF et de poursuivre avec les autres fichiers
sélectionnés.

## Périmètre de la première version

La première version couvre uniquement les trois fonctions suivantes :

- fusionner des PDF ;
- scinder des PDF ;
- masquer définitivement des informations dans un PDF.

Aucun autre besoin fonctionnel n'est demandé à ce stade.

## Décisions fonctionnelles à prendre

Les points suivants ne sont pas encore définis et ne doivent pas être
interprétés comme des fonctionnalités décidées :

_Aucune à ce stade. Toute nouvelle décision fera l'objet d'une validation de
l'utilisateur final avant d'être ajoutée._

## Fonctionnalités à définir ultérieurement

Les fonctionnalités futures seront ajoutées à cette section seulement après
validation explicite de l'utilisateur final.
