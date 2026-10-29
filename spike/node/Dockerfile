FROM rust:1.88-bookworm AS builder
WORKDIR /workspace
# Contexte de build = spike/ (workspace Cargo complet)
COPY . .
RUN cargo build --release -p node

FROM debian:bookworm-slim
# Client PostgreSQL 17 (pg_dump / pg_restore) pour les sauvegardes chiffrées
# (refonte V3, étape 4) : il doit être de la version du serveur (Spilo 17), que
# Debian bookworm ne fournit pas — dépôt officiel PostgreSQL (PGDG), clé vérifiée.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 curl gnupg \
 && install -d /usr/share/postgresql-common/pgdg \
 && curl -fsSL -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc https://www.postgresql.org/media/keys/ACCC4CF8.asc \
 && echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] https://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" > /etc/apt/sources.list.d/pgdg.list \
 && apt-get update \
 && apt-get install -y --no-install-recommends postgresql-client-17 \
 && apt-get purge -y curl gnupg && apt-get autoremove -y \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/ss-node /usr/local/bin/ss-node
ENV SS_NODE_DIR=/data
VOLUME ["/data"]
EXPOSE 9001

# Script d'entrée : init --first si premier serveur (génère la DEK), init seul
# sinon (le serveur attend son enrôlement : `ss-node enroler`).
RUN printf '#!/bin/sh\nset -e\nif [ ! -f "$SS_NODE_DIR/config.toml" ]; then\n  if [ "${NODE_MODE}" = "active" ]; then\n    ss-node init --first\n  else\n    ss-node init\n  fi\nfi\nexec ss-node "$@"\n' > /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["run", "--mode", "passive"]
