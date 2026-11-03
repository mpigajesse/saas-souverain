# Image app-migrations : SaaS externe avec hook pre_start + migration SQL
# AMANE NE MODIFIE PAS cette image (construite par l'éditeur)
# Multi-stage : build Go -> final alpine (pour Phase 1 test, Phase 3 = scratch)

# ---- Build stage ----
FROM golang:1.23-alpine AS build

# Installer outils pour migrations (psql, pg_isready)
RUN apk add --no-cache postgresql-client

WORKDIR /src

# Dépendances Go
COPY go.mod go.sum ./
RUN go mod download

# Sources
COPY . .

# Build statique
RUN CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o /application .

# ---- Final stage : Alpine minimal (pour test Phase 1) ----
# Phase 3 : FROM scratch + copies minimales
FROM alpine:3.22

# Installer runtime deps pour psql/pg_isready
RUN apk add --no-cache postgresql-client

WORKDIR /app

# Copier artefacts depuis build
COPY --from=build /application /application
COPY --from=build /src/pre_start.sh /pre_start.sh
COPY --from=build /src/migrations /migrations

# Rendre exécutable
RUN chmod +x /pre_start.sh /application

EXPOSE 8080

# WORKAROUND Phase 1 : hook pre_start appelé via commande shell
# Phase 3 : le lanceur AMANE appellera preStart nativement
ENTRYPOINT ["/bin/sh", "-c"]
CMD ["/pre_start.sh && /application"]