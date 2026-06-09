FROM rust:1.85-slim AS builder
WORKDIR /app
# Copier le workspace entier pour que Cargo résolve les dépendances locales
COPY . .
RUN cargo build --release -p relay

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/ss-relay /usr/local/bin/ss-relay
ENV RELAY_PORT=8080
EXPOSE 8080
CMD ["ss-relay"]
