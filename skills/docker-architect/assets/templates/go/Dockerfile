# syntax=docker/dockerfile:1

ARG GO_VERSION=1.25
ARG DEBIAN_RELEASE=bookworm

FROM golang:${GO_VERSION}-${DEBIAN_RELEASE} AS build
WORKDIR /src

COPY go.mod go.sum ./
RUN --mount=type=cache,target=/go/pkg/mod \
  go mod download

COPY . .
ARG GO_MAIN=.
RUN --mount=type=cache,target=/root/.cache/go-build \
  CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o /out/app "${GO_MAIN}"

FROM gcr.io/distroless/static-debian12:nonroot AS runtime
WORKDIR /
COPY --from=build /out/app /app

EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/app"]
