# FTR-006 — Mettre PDFForge à jour

## Statut et source de vérité

Cette fiche précise la fonctionnalité de mise à jour confirmée par
l'utilisateur final. En cas de divergence, `specs/SFG.md` prévaut toujours.

## Intention

Permettre à l'utilisateur d'obtenir une version stable plus récente de
PDFForge sans télécharger ni remplacer manuellement l'application lorsque son
emplacement est inscriptible.

## Accès et vérification volontaire

Depuis l'écran d'accueil, un lien discret affiche la version installée de
PDFForge parmi les préférences. Il ouvre la fenêtre « À propos et mises à
jour ». Cette fenêtre affiche la version installée et propose « Rechercher les
mises à jour ».

L'application ne vérifie, ne télécharge et n'installe jamais une mise à jour au
démarrage, à intervalle régulier ou en arrière-plan. Les fonctions de traitement
PDF restent utilisables sans connexion Internet.

La recherche demandée par l'utilisateur consulte exclusivement les GitHub
Releases publiques du projet par HTTPS. Sont éligibles uniquement les Releases
publiées, non brouillons et non préversions, dont le tag est de la forme
`vX.Y.Z`. Une Release est proposée dès sa publication. L'application compare
localement les versions selon leur numéro sémantique ; elle n'affiche jamais une
version identique ou plus ancienne et ne propose ni alpha, ni bêta, ni autre
préversion.

La requête de vérification n'ajoute ni la version locale, ni la plateforme, ni
la langue, ni un identifiant d'installation. Ces données servent uniquement
dans l'application. GitHub reçoit toutefois l'adresse IP nécessaire au transport
réseau, sans que PDFForge ne l'exploite.

## Résultat de la recherche

Sans nouvelle version, l'application affiche « PDFForge est à jour. ».

Lorsqu'une Release ne contient pas d'artefact pour la plateforme courante,
l'application affiche « Cette version n'est pas disponible pour votre système. ».

Lorsqu'une nouvelle version est disponible, l'application affiche son numéro et
ses notes de version. Chaque Release fournit des notes françaises et anglaises.
Les notes françaises sont affichées lorsque la langue de l'utilisateur est le
français ; les notes anglaises le sont dans tous les autres cas.

L'utilisateur peut alors choisir « Télécharger et installer » ou abandonner la
mise à jour. Un échec de recherche ou de téléchargement est affiché par un
message simple ; l'utilisateur doit relancer lui-même une recherche, car aucune
nouvelle tentative automatique n'est faite.

## Télécharger et vérifier

Le téléchargement commence seulement après la confirmation « Télécharger et
installer ». L'application affiche son avancement, peut être annulée par
l'utilisateur et supprime alors le fichier partiel.

Chaque artefact de mise à jour est signé cryptographiquement et accompagné d'une
somme de contrôle. Avant l'installation, l'application vérifie obligatoirement
la signature et la somme de contrôle de chaque fichier. En cas d'échec de l'une
de ces vérifications, elle supprime les fichiers téléchargés et affiche un
message simple. En cas d'espace disque insuffisant, elle signale une erreur et
ne conserve aucun fichier partiel.

Les artefacts de chaque plateforme comprennent tous les composants de PDFForge,
y compris le moteur PDF embarqué.

## Installer et redémarrer

Une installation ne peut pas commencer pendant un traitement PDF actif. Dans ce
cas, l'application avertit l'utilisateur et lui permet d'annuler la fermeture
pour revenir au traitement. Une préparation non encore exécutée peut être perdue
au redémarrage.

Si l'application a été déplacée, renommée ou supprimée pendant le téléchargement,
ou si une autre instance de PDFForge empêche son remplacement, l'installation
est annulée, les fichiers téléchargés sont supprimés et l'utilisateur est invité
à recommencer après avoir résolu le problème.

Lorsque l'emplacement de l'application est inscriptible, PDFForge remplace la
version utilisée au même emplacement et redémarre immédiatement. Les fichiers
temporaires sont supprimés après une installation réussie, une annulation ou un
échec de vérification.

Lorsque cet emplacement n'est pas inscriptible, PDFForge télécharge la nouvelle
version dans le dossier Téléchargements de l'utilisateur, ouvre ce dossier et
laisse l'utilisateur remplacer l'application manuellement.

Après un redémarrage réussi, PDFForge affiche « PDFForge a été mis à jour vers
la version X. ».

## Artefacts par plateforme

Sous Linux, la mise à jour est une AppImage.

Sous Windows, elle est un unique exécutable autonome, sans archive ZIP. Cet
exécutable extrait ses composants techniques dans le dossier local de
l'utilisateur. Si ces composants sont supprimés, il les recrée depuis lui-même
lorsque cela est possible. Ces composants techniques ne constituent ni un
historique de PDF ni une télémétrie.

## Restaurer une version antérieure

PDFForge conserve une seule version antérieure dans un sous-dossier local, avec
ses composants extraits. La fenêtre « À propos et mises à jour » propose
« Restaurer la version précédente » lorsque cette sauvegarde existe.

Cette action remplace la version courante, redémarre immédiatement la version
antérieure et supprime définitivement la version abandonnée. La version
antérieure est conservée après une mise à jour réussie afin que cette
restauration reste possible.

## Confidentialité et absence d'historique

PDFForge ne crée aucun identifiant persistant, ne transmet aucune télémétrie et
ne conserve aucun journal ni historique de mise à jour. La conservation de la
seule version antérieure et des composants techniques Windows est limitée au
fonctionnement de la mise à jour et ne constitue pas un historique de documents
traités.

## Exigences SFG couvertes

- F-03, F-24, F-26, F-37, F-39 à F-46 ;
- U-01 à U-04 ;
- parcours « Mettre PDFForge à jour ».
