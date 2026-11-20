# Dockerfile commun à tous les services du monorepo.
#   docker build --build-arg SERVICE=avis-etudiants -t avis-etudiants .
# Convention : chaque service écoute sur $BIND_ADDR et répond « ok » sur /healthz.

# ---- Compilation
FROM rust:1-bookworm AS build
ARG SERVICE
RUN test -n "$SERVICE" || (echo "SERVICE manquant (--build-arg SERVICE=…)" && false)
RUN apt-get update && apt-get install -y --no-install-recommends cmake && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked -p "$SERVICE" && cp "target/release/$SERVICE" /service

# ---- Exécution
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --uid 10001 app
COPY --from=build /service /usr/local/bin/service
USER app
ENV BIND_ADDR=0.0.0.0:8080 RUST_LOG=info,sqlx=warn
EXPOSE 8080
# bash (et non sh/dash) : nécessaire pour /dev/tcp
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/bin/bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/${BIND_ADDR##*:} && printf 'GET /healthz HTTP/1.0\\r\\n\\r\\n' >&3 && grep -q ok <&3"]
CMD ["service"]
