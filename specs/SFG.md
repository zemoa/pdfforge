# Spécification fonctionnelle générale — PDFForge

## Statut du document

Spécification fonctionnelle générale mise à jour le 6 septembre 2026. Elle ne
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
| F-22      | Que le texte d'un PDF soit sélectionnable ou non, l'utilisateur doit pouvoir dessiner un rectangle pour masquer définitivement une zone de la page, notamment une image ou une icône.                                     |
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
| F-40      | L'utilisateur doit pouvoir rechercher volontairement une mise à jour stable de PDFForge, publiée dans les GitHub Releases publiques du projet.                                                                            |
| F-41      | L'application ne doit jamais vérifier ni télécharger une mise à jour sans une demande explicite de l'utilisateur. Les fonctions PDF restent utilisables sans connexion Internet.                                          |
| F-42      | L'application doit afficher discrètement sa version installée et indiquer clairement si une version stable plus récente est disponible.                                                                                   |
| F-43      | Après confirmation de l'utilisateur, l'application doit télécharger, vérifier, installer et redémarrer vers la mise à jour sans téléchargement ou installation manuels lorsque son emplacement est inscriptible.          |
| F-44      | Avant toute installation, l'application doit vérifier la signature cryptographique et la somme de contrôle de chaque fichier de mise à jour ; en cas d'échec, elle doit supprimer les fichiers téléchargés.               |
| F-45      | L'application ne doit transmettre aucune télémétrie, aucun identifiant persistant ni historique de mise à jour.                                                                                                           |
| F-46      | L'application doit conserver une seule version antérieure afin que l'utilisateur puisse la restaurer ; une restauration redémarre l'ancienne version et supprime la version abandonnée.                                   |

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
glisser-déposer. Que le texte soit sélectionnable ou non, il peut dessiner un
rectangle sur une zone à masquer, notamment une image ou une icône. Il peut afficher la page en grand, zoomer et
préparer plusieurs zones noires à masquer. Après confirmation, l'application crée un
nouveau PDF sans les informations masquées, qui ne doivent pas pouvoir être
récupérées. Avant confirmation, il peut vérifier, modifier ou retirer les
zones sélectionnées. Le document d'origine reste intact. Par défaut, le nom du
nouveau fichier est `<nom du document d'origine>-masked.pdf` et son dossier de
destination est celui du document d'origine ; l'utilisateur peut modifier ces
deux valeurs avant confirmation.

### Gérer les incidents

Si un PDF protégé par mot de passe ou illisible est rencontré, l'application ne
demande pas de mot de passe. Elle laisse à l'utilisateur le choix d'arrêter
l'opération ou d'ignorer ce PDF et de poursuivre avec les autres fichiers
sélectionnés.

### Mettre PDFForge à jour

L'utilisateur accède à une fenêtre discrète « À propos et mises à jour » depuis
un lien affichant la version de PDFForge dans les préférences de l'écran
d'accueil. Cette fenêtre affiche la version installée et propose la commande
« Rechercher les mises à jour ». Aucune vérification n'a lieu au démarrage, à
intervalle régulier ou en arrière-plan.

À la demande de l'utilisateur, l'application consulte exclusivement les GitHub
Releases publiques du projet par HTTPS. Seules les Releases publiées, non
brouillons et non préversions, dont le tag respecte le format `vX.Y.Z`, sont
éligibles. Une Release devient disponible dès sa publication. L'application
compare localement les versions selon leur numéro sémantique et ne propose
jamais une version identique ou plus ancienne. La requête de vérification
n'ajoute ni version locale, ni plateforme, ni langue, ni identifiant
d'installation : ces informations servent uniquement dans l'application. Toute
utilisation de GitHub implique néanmoins la transmission de l'adresse IP par le
transport réseau, sans exploitation par PDFForge.

Si aucune version n'est disponible, l'application affiche « PDFForge est à
jour. ». Si aucune version n'est fournie pour la plateforme courante, elle
affiche « Cette version n'est pas disponible pour votre système. ». Si une
version est disponible, l'application affiche son numéro et ses notes. Chaque
Release fournit des notes françaises et anglaises ; les notes françaises sont
affichées lorsque la langue de l'utilisateur est le français, et les notes
anglaises dans tous les autres cas. Aucun canal alpha, bêta ou préversion n'est
proposé.

L'utilisateur confirme le téléchargement par « Télécharger et installer ».
Il peut l'annuler ; l'application supprime alors le fichier partiel. Elle
affiche l'avancement du téléchargement et de l'installation. Elle ne réessaie
jamais automatiquement un échec : elle affiche un message simple et
l'utilisateur relance lui-même la recherche. Un manque d'espace disque est
signalé comme une erreur et aucun fichier partiel n'est conservé.

Les artefacts téléchargés doivent être signés cryptographiquement et accompagnés
d'une somme de contrôle. L'application vérifie les deux avant l'installation et
supprime le téléchargement en cas d'échec. Les mises à jour Linux et Windows
comprennent tous les composants de PDFForge, y compris le moteur PDF embarqué.

Sur Linux, l'artefact est une AppImage. Sur Windows, il s'agit d'un unique
exécutable autonome qui extrait ses composants techniques dans le dossier local
de l'utilisateur ; il recrée ces composants depuis lui-même s'ils ont été
supprimés, lorsque cela est possible. Ces fichiers techniques ne constituent ni
un historique de PDF ni une télémétrie.

Si l'emplacement de l'application est inscriptible, l'installation remplace la
version utilisée au même emplacement et relance immédiatement PDFForge. Si cet
emplacement n'est pas inscriptible, l'application télécharge la nouvelle version
dans le dossier Téléchargements de l'utilisateur, ouvre ce dossier et laisse
l'utilisateur remplacer l'application manuellement. Les fichiers temporaires
sont supprimés après une installation réussie, une annulation ou un échec de
vérification.

Une installation est refusée pendant un traitement PDF actif, avec un
avertissement permettant d'annuler la fermeture et de revenir au traitement.
Une préparation de traitement non encore lancée peut être perdue lors du
redémarrage. Si une autre instance de PDFForge empêche le remplacement, ou si
l'application a été déplacée, renommée ou supprimée pendant le téléchargement,
l'installation est annulée, les fichiers téléchargés sont supprimés et
l'utilisateur est invité à recommencer après avoir résolu le problème.

PDFForge conserve dans un sous-dossier local une unique version antérieure,
avec ses composants extraits. La fenêtre « À propos et mises à jour » propose
« Restaurer la version précédente ». Cette action remplace la version courante,
redémarre immédiatement l'ancienne version et supprime définitivement la version
abandonnée. Après une mise à jour réussie suivie d'un redémarrage, l'application
affiche « PDFForge a été mis à jour vers la version X. » et conserve toujours la
version antérieure pour une restauration manuelle ultérieure.

## Périmètre de la première version

La première version couvre uniquement les trois fonctions suivantes :

- fusionner des PDF ;
- scinder des PDF ;
- masquer définitivement des informations dans un PDF.

Le mécanisme de mise à jour défini ci-dessus fait partie de la première version
publiée, mais n'est pas une fonction de traitement PDF.

Aucun autre besoin fonctionnel n'est demandé à ce stade.

## Décisions fonctionnelles à prendre

Les points suivants ne sont pas encore définis et ne doivent pas être
interprétés comme des fonctionnalités décidées :

_Aucune à ce stade. Toute nouvelle décision fera l'objet d'une validation de
l'utilisateur final avant d'être ajoutée._

## Fonctionnalités à définir ultérieurement

Les fonctionnalités futures seront ajoutées à cette section seulement après
validation explicite de l'utilisateur final.
