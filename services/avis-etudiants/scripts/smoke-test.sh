#!/usr/bin/env bash
# Test de bout en bout des droits, sur un service lancé avec la feature `dev-login` et DEV_LOGIN=1.
# Usage : BASE=http://localhost:8080 SUPER=chef@normandiewebschool.fr ./scripts/smoke-test.sh
set -u
BASE=${BASE:-http://localhost:8080}
SUPER=${SUPER:-chef@normandiewebschool.fr}
DOM=normandiewebschool.fr
TMP=$(mktemp -d)
FAIL=0
STAMP=$(date +%s)
ETU="etu$STAMP@$DOM"; AUTRE="autre$STAMP@$DOM"; ADM="admin$STAMP@$DOM"
TODAY=$(TZ=Europe/Paris date +%F)

login() { curl -s -o /dev/null -c "$TMP/$1" "$BASE/dev/login?email=$2"; }
# csrf <jar> : extrait le jeton CSRF de la page /tickets/new
csrf() { curl -s -b "$TMP/$1" "$BASE/tickets/new" | grep -o 'name="csrf" value="[0-9a-f]*"' | head -1 | sed 's/.*value="//;s/"//'; }
code() { local jar=$1; shift; curl -s -o /dev/null -w '%{http_code}' -b "$TMP/$jar" "$@"; }
check() { if [ "$2" = "$3" ]; then echo "  ok   $1"; else echo "  FAIL $1 (attendu $3, obtenu $2)"; FAIL=1; fi; }
contains() { if curl -s -b "$TMP/$2" "$3" | grep -q "$4"; then r=yes; else r=no; fi; check "$1" "$r" "${5:-yes}"; }

echo "== Connexion"
check "page d'accueil anonyme" "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/")" 200
check "/tickets sans session → redirection" "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/tickets")" 303
check "adresse hors domaine refusée" "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/dev/login?email=x@gmail.com")" 400
login etu "$ETU"; login autre "$AUTRE"; login super "$SUPER"
T_ETU=$(csrf etu); T_AUTRE=$(csrf autre); T_SUPER=$(csrf super)
check "jeton CSRF présent" "$([ ${#T_ETU} -eq 64 ] && echo yes)" yes

echo "== Création"
check "sans CSRF → refus" "$(code etu -X POST -d "date=$TODAY&category=Cours&rating=3" "$BASE/tickets")" 400
check "formulaire invalide → 422" "$(code etu -X POST -d "csrf=$T_ETU&date=2099-01-01&category=Inconnue&rating=9" "$BASE/tickets")" 422
check "création OK → redirection" "$(code etu -X POST -d "csrf=$T_ETU&date=$TODAY&category=Cours&teacher=M.+Dupont&rating=4&comment=Tr%C3%A8s+bien+%3Cb%3Ex%3C%2Fb%3E" "$BASE/tickets")" 303
code etu -X POST -d "csrf=$T_ETU&date=$TODAY&category=Autre&rating=1" "$BASE/tickets" >/dev/null
IDS=$(curl -s -b "$TMP/etu" "$BASE/tickets" | grep -o 'href="/tickets/[0-9]*"' | grep -o '[0-9]*' | sort -n)
T1=$(echo "$IDS" | head -1); T2=$(echo "$IDS" | tail -1)
check "deux tickets listés" "$(echo "$IDS" | wc -l | tr -d ' ')" 2
contains "HTML échappé (pas d'injection)" etu "$BASE/tickets/$T1" '&#60;b&#62;x&#60;/b&#62;'
contains "pas de balise <b> brute" etu "$BASE/tickets/$T1" '<b>x</b>' no

echo "== Cloisonnement étudiant"
check "autre étudiant ne voit pas le ticket" "$(code autre "$BASE/tickets/$T1")" 404
check "étudiant n'accède pas à l'admin" "$(code etu "$BASE/admin/tickets")" 403
check "étudiant ne peut pas répondre" "$(code etu -X POST -d "csrf=$T_ETU&reply=x" "$BASE/admin/tickets/$T1/reply")" 403
check "autre étudiant ne peut pas annuler" "$(code autre -X POST -d "csrf=$T_AUTRE" "$BASE/tickets/$T1/cancel")" 404

echo "== Superadmin : gestion des admins"
check "étudiant n'accède pas à /admin/admins" "$(code etu "$BASE/admin/admins")" 403
check "ajout admin hors domaine refusé" "$(code super -X POST -d "csrf=$T_SUPER&email=x@gmail.com" "$BASE/admin/admins")" 422
check "ajout admin" "$(code super -X POST -d "csrf=$T_SUPER&email=$ADM" "$BASE/admin/admins")" 303
login adm "$ADM"; T_ADM=$(csrf adm)
check "admin accède à la liste" "$(code adm "$BASE/admin/tickets")" 200
check "admin n'accède pas à /admin/admins" "$(code adm "$BASE/admin/admins")" 403
contains "admin voit le ticket de l'étudiant" adm "$BASE/admin/tickets/$T1" "$ETU"

echo "== Traitement"
check "réponse vide refusée" "$(code adm -X POST -d "csrf=$T_ADM&reply=+" "$BASE/admin/tickets/$T1/reply")" 422
check "admin répond" "$(code adm -X POST -d "csrf=$T_ADM&reply=Merci+pour+ce+retour" "$BASE/admin/tickets/$T1/reply")" 303
contains "l'étudiant voit la réponse" etu "$BASE/tickets/$T1" "Merci pour ce retour"
check "annulation impossible une fois répondu" "$(code etu -X POST -d "csrf=$T_ETU" "$BASE/tickets/$T1/cancel")" 400
check "statut en_cours" "$(code adm -X POST -d "csrf=$T_ADM&status=en_cours" "$BASE/admin/tickets/$T1/status")" 303
check "statut annule interdit à l'admin" "$(code adm -X POST -d "csrf=$T_ADM&status=annule" "$BASE/admin/tickets/$T1/status")" 400
check "statut ferme" "$(code adm -X POST -d "csrf=$T_ADM&status=ferme" "$BASE/admin/tickets/$T1/status")" 303
contains "historique tracé" adm "$BASE/admin/tickets/$T1" "En cours → Fermé"

echo "== Annulation par l'étudiant"
check "annulation d'un ticket non traité" "$(code etu -X POST -d "csrf=$T_ETU" "$BASE/tickets/$T2/cancel")" 303
contains "statut annulé visible" etu "$BASE/tickets/$T2" "Annulé par l"
check "admin ne peut plus changer le statut" "$(code adm -X POST -d "csrf=$T_ADM&status=ouvert" "$BASE/admin/tickets/$T2/status")" 400
check "admin ne peut plus répondre" "$(code adm -X POST -d "csrf=$T_ADM&reply=x" "$BASE/admin/tickets/$T2/reply")" 400

echo "== Suppression"
check "admin ne peut pas supprimer" "$(code adm -X POST -d "csrf=$T_ADM" "$BASE/admin/tickets/$T1/delete")" 403
check "superadmin supprime" "$(code super -X POST -d "csrf=$T_SUPER" "$BASE/admin/tickets/$T1/delete")" 303
check "ticket supprimé invisible (étudiant)" "$(code etu "$BASE/tickets/$T1")" 404
check "ticket supprimé invisible (admin)" "$(code adm "$BASE/admin/tickets/$T1")" 404
check "ticket supprimé absent de « mes tickets »" "$(curl -s -b "$TMP/etu" "$BASE/tickets" | grep -c "href=\"/tickets/$T1\"")" 0

echo "== Retrait d'un admin (effet immédiat)"
check "retrait" "$(code super -X POST -d "csrf=$T_SUPER&email=$ADM" "$BASE/admin/admins/remove")" 303
check "l'ancien admin perd l'accès" "$(code adm "$BASE/admin/tickets")" 403

echo "== Déconnexion"
check "déconnexion" "$(code etu -c "$TMP/etu" -X POST -d "csrf=$T_ETU" "$BASE/logout")" 303
check "session fermée" "$(code etu "$BASE/tickets")" 303

rm -rf "$TMP"
[ $FAIL = 0 ] && echo "TOUT EST OK" || { echo "ÉCHECS"; exit 1; }
