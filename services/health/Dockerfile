# syntax=docker/dockerfile:1
# Image du service health. Contexte de build : racine du dépôt (workspace Cargo).
#   docker build -f services/health/Dockerfile -t edusense/health .

FROM rust:1.97.1-slim-bookworm AS builder
# Toolchain de l'image (même version que rust-toolchain.toml, non copié) : pas de
# téléchargement des composants rustfmt/clippy, inutiles au build.
ENV RUSTUP_TOOLCHAIN=1.97.1
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY services ./services
# Caches BuildKit du registre et du répertoire target : seules les crates modifiées sont
# recompilées. Le binaire est copié hors du cache pour l'étape suivante.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked -p health-service \
    && cp target/release/health-service /usr/local/bin/health-service

# Image d'exécution minimale, sans shell, en utilisateur non root.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/local/bin/health-service /usr/local/bin/health-service
ENV HEALTH_BIND_ADDR=0.0.0.0:8080 \
    RUST_LOG=info
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/usr/local/bin/health-service"]
