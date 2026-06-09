FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p node

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ss-node /usr/local/bin/ss-node
ENV SS_NODE_DIR=/data
VOLUME ["/data"]
EXPOSE 9001

# Script d'entrée qui initialise le nœud si nécessaire avant de le démarrer
RUN echo '#!/bin/sh\nset -e\nif [ ! -f "$SS_NODE_DIR/config.toml" ]; then\n  ss-node init\nfi\nexec ss-node "$@"' > /entrypoint.sh && chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["run", "--mode", "passive"]
