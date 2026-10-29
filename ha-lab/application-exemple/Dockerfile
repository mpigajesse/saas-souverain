# Image d'un « éditeur externe » : AMANE ne la modifie pas.
FROM golang:1.26-alpine AS build
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o /application .

FROM scratch
COPY --from=build /application /application
EXPOSE 8080
ENTRYPOINT ["/application"]
