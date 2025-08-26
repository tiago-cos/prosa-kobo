# setup cargo-chef
FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

# generate recipe for caching
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# build prosa-kobo with cached dependencies
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin prosa-kobo

FROM alpine AS runtime

# setup a healthcheck
HEALTHCHECK --interval=300s --timeout=5s --retries=3 --start-period=10s \
  CMD wget --spider -q http://127.0.0.1:${SERVER__BIND__PORT:-5001}/health || exit 1

# copy binaries
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/prosa-kobo /usr/local/bin/

# setup default config file
RUN mkdir -p /app/config
COPY --from=builder /app/src/config/default.toml /app/config/default.toml
ENV DEFAULT_CONFIGURATION=/app/config/default.toml

# run prosa-kobo as non-root user
RUN mkdir /app/persistence
RUN addgroup -S prosa-kobo && adduser -S prosa-kobo -G prosa-kobo
RUN chown -R prosa-kobo:prosa-kobo /app
USER prosa-kobo
WORKDIR /app

ENTRYPOINT ["sh", "-c", "\
    unset DATABASE__FILE_PATH; \
    exec /usr/local/bin/prosa-kobo \
"]
