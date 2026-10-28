# Image de l'orchestrateur Go (Mission C) — superviseur de failover Patroni,
# consensus etcd, serveur gRPC du contrat proto/amane/framework/v1.
#
# Contexte de build : le dossier orchestrator-go/ (module Go autonome, bindings
# gRPC déjà générés dans gen/).
#   docker build -t amane-orchestrator:dev orchestrator-go

FROM golang:1.26-alpine AS build
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
# Binaire statique : l'image finale n'embarque ni libc ni toolchain.
RUN CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o /out/orchestrator ./cmd/orchestrator

FROM alpine:3.20
# Compte non privilégié : l'orchestrateur n'a besoin d'aucun droit root
# (etcd et Patroni sont joints par le réseau).
RUN adduser -D -H -u 10001 amane
COPY --from=build /out/orchestrator /usr/local/bin/orchestrator
USER amane
EXPOSE 50051
ENTRYPOINT ["/usr/local/bin/orchestrator"]
