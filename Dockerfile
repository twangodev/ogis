FROM rust:1.90 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="ogis" \
      org.opencontainers.image.description="A fast, free, and beautiful platform for open graph image generation" \
      org.opencontainers.image.source="https://github.com/twangodev/ogis" \
      org.opencontainers.image.licenses="AGPL-3.0"

RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -r -s /bin/false ogis

WORKDIR /app

COPY --from=builder /app/target/release/ogis .

# Include default templates and fonts in the image. They can be overridden by mounting volumes.
COPY fonts ./fonts
COPY templates ./templates
COPY fonts.yaml ./fonts.yaml
COPY templates.yaml ./templates.yaml

RUN chown -R ogis:ogis /app

USER ogis

EXPOSE 3000

HEALTHCHECK --interval=10s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./ogis"]