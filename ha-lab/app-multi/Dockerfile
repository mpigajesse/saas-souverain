# Image app-multi : SaaS multi-services (Web + Worker Redis)
# Multi-stage : build Go -> alpine runtime (Phase 1)

# ---- Build stage ----
FROM golang:1.23-alpine AS build

RUN apk add --no-cache postgresql-client

WORKDIR /src

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o /application .

# ---- Final stage : Alpine minimal ----
FROM alpine:3.22

RUN apk add --no-cache postgresql-client

WORKDIR /app

COPY --from=build /application /application
COPY --from=build /src/pre_start.sh /pre_start.sh
COPY --from=build /src/migrations /migrations

RUN chmod +x /pre_start.sh /application

EXPOSE 8080

# WORKAROUND Phase 1 : pre_start via shell
# Mode choisi via variable d'env MODE (web|worker) ou argument
ENTRYPOINT ["/bin/sh", "-c"]
CMD ["/pre_start.sh && /application --mode ${MODE:-web} --port ${PORT:-8080} --redis ${REDIS_URL:-redis://redis:6379}"]