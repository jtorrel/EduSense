# Image du service health. Contexte de build : racine du dépôt (workspace Cargo).
#   docker build -f services/health/Dockerfile -t edusense/health .

FROM rust:1.97.1-slim-bookworm AS builder
WORKDIR /app
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY services ./services
RUN cargo build --release --locked -p health-service

# Image d'exécution minimale, sans shell, en utilisateur non root.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/health-service /usr/local/bin/health-service
ENV HEALTH_BIND_ADDR=0.0.0.0:8080 \
    RUST_LOG=info
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/usr/local/bin/health-service"]
