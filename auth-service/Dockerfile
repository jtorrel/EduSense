# Stage 1 — Compilation
FROM rust:1.85-slim AS builder

WORKDIR /app

# Copie des fichiers de dépendances
COPY Cargo.toml Cargo.lock ./
COPY auth-service/Cargo.toml ./auth-service/

# Pré-compilation des dépendances (optimisation cache Docker)
RUN mkdir -p auth-service/src && \
    echo "fn main() {}" > auth-service/src/main.rs && \
    cargo build --release --package edusense-auth && \
    rm -rf auth-service/src

# Copie du vrai code source
COPY auth-service/src ./auth-service/src

# Compilation finale
RUN touch auth-service/src/main.rs && \
    cargo build --release --package edusense-auth

# Stage 2 — Image finale
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Copie du binaire uniquement
COPY --from=builder /app/target/release/edusense-auth .

EXPOSE 3000

CMD ["./edusense-auth"]