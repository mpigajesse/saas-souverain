# Démo publique du logiciel PME (nœud ss-node) — un seul conteneur, gratuit sur Render.
#
# Contrairement au déploiement réel (pme-deploy/ : PostgreSQL primaire/standby sur
# les machines de la PME), la démo embarque un PostgreSQL local ÉPHÉMÈRE : à chaque
# réveil du service, la base est recréée avec des articles fictifs. Le relais n'est
# pas joignable en démo : le nœud le signale et continue (comportement du spike).
#
# Build : contexte = spike/   →   docker build -f demo/Dockerfile .
FROM rust:1.88-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo build --release -p node

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 postgresql postgresql-contrib \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/ss-node /usr/local/bin/ss-node
COPY demo/entrypoint.sh /entrypoint.sh
COPY demo/seed.sql /seed.sql
RUN chmod +x /entrypoint.sh
ENV SS_NODE_DIR=/data NODE_MODE=active
ENTRYPOINT ["/entrypoint.sh"]
