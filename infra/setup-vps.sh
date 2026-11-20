#!/usr/bin/env bash
# Installation d'un VPS Ubuntu 26.04 pour les services EduSense.
#
#   Depuis votre poste :  scp -r edusense-infra ubuntu@IP_DU_VPS:
#   Puis :                ssh ubuntu@IP_DU_VPS
#                         sudo bash edusense-infra/setup-vps.sh
#
# Le script peut être relancé sans risque (par exemple pour mettre à jour l'outil « edusense »).
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ADMIN_USER=${SUDO_USER:-}
step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
die()  { echo "✗ $*" >&2; exit 1; }

[ "$(id -u)" = 0 ] || die "lancez le script avec sudo"
[ -n "$ADMIN_USER" ] && [ "$ADMIN_USER" != root ] || die "lancez-le avec sudo depuis votre utilisateur (ex. ubuntu), pas en root direct"
for f in edusense socle/docker-compose.yml socle/Caddyfile; do
  [ -f "$HERE/$f" ] || die "fichier manquant : $HERE/$f (copiez tout le dossier edusense-infra)"
done
# Garde-fou : on ne coupe pas l'accès par mot de passe si aucune clé SSH n'est installée.
[ -s "/home/$ADMIN_USER/.ssh/authorized_keys" ] || die "aucune clé SSH pour $ADMIN_USER : arrêt, pour ne pas vous bloquer dehors"

export DEBIAN_FRONTEND=noninteractive

step "Mise à jour du système"
apt-get update -q
apt-get -y -q upgrade
apt-get -y -q install ca-certificates curl ufw unattended-upgrades openssl gzip

step "Mises à jour de sécurité automatiques"
cat > /etc/apt/apt.conf.d/20auto-upgrades <<'EOF'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
EOF

step "SSH : connexion par clé uniquement, pas de root"
# « 00- » : lu avant le fichier d'OVH/cloud-init, c'est donc notre réglage qui s'applique.
cat > /etc/ssh/sshd_config.d/00-edusense.conf <<'EOF'
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitRootLogin no
EOF
mkdir -p /run/sshd
sshd -t || die "configuration SSH invalide"
systemctl try-reload-or-restart ssh

step "Pare-feu : SSH, HTTP et HTTPS uniquement"
ufw allow OpenSSH >/dev/null
ufw allow 80/tcp >/dev/null
ufw allow 443/tcp >/dev/null
ufw allow 443/udp >/dev/null
ufw --force enable >/dev/null
ufw status | sed 's/^/    /'

step "Docker"
if ! command -v docker >/dev/null; then
  install -m 0755 -d /etc/apt/keyrings
  curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
  chmod a+r /etc/apt/keyrings/docker.asc
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
    > /etc/apt/sources.list.d/docker.list
  apt-get update -q
  apt-get -y -q install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
fi
# Journaux limités en taille, pour ne pas remplir le disque.
if [ ! -f /etc/docker/daemon.json ]; then
  cat > /etc/docker/daemon.json <<'EOF'
{ "log-driver": "json-file", "log-opts": { "max-size": "10m", "max-file": "5" } }
EOF
  systemctl restart docker
fi
systemctl enable --now docker >/dev/null
usermod -aG docker "$ADMIN_USER"
docker network inspect edusense >/dev/null 2>&1 || docker network create edusense >/dev/null
docker --version | sed 's/^/    /'

step "Socle : Caddy (HTTPS) + MySQL 8"
mkdir -p /srv/socle/sites /srv/backups
install -m 644 "$HERE/socle/docker-compose.yml" /srv/socle/docker-compose.yml
install -m 644 "$HERE/socle/Caddyfile" /srv/socle/Caddyfile
if [ ! -f /srv/socle/.env ]; then
  echo "MYSQL_ROOT_PASSWORD=$(openssl rand -hex 24)" > /srv/socle/.env
fi
chmod 600 /srv/socle/.env
chown -R "$ADMIN_USER:$ADMIN_USER" /srv
chmod 700 /srv/backups
(cd /srv/socle && docker compose up -d --quiet-pull)

step "Outil « edusense »"
install -m 755 "$HERE/edusense" /usr/local/bin/edusense

step "Sauvegarde automatique des bases (chaque nuit à 3 h 30, 14 jours conservés)"
cat > /etc/cron.d/edusense-backup <<'EOF'
30 3 * * * root /usr/local/bin/edusense backup >> /var/log/edusense-backup.log 2>&1
EOF

step "Attente de MySQL"
for _ in $(seq 1 60); do
  docker compose -f /srv/socle/docker-compose.yml ps mysql --format '{{.Status}}' | grep -q '(healthy)' && break
  sleep 3
done
edusense status

cat <<EOF

✓ Serveur prêt.

  1. Déconnectez-vous puis reconnectez-vous (pour utiliser docker sans sudo).
  2. Connectez le serveur à GitHub (une seule fois) :
       docker login ghcr.io -u VOTRE_PSEUDO_GITHUB
     (mot de passe = un jeton GitHub avec le droit « read:packages »)
  3. Créez votre premier service : edusense help
EOF
