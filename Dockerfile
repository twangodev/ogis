FROM rust:1.90 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ogis .

# Include default templates and fonts in the image. They can be overridden by mounting volumes.
COPY fonts ./fonts
COPY templates ./templates
COPY fonts.yaml ./fonts.yaml
COPY templates.yaml ./templates.yaml

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./ogis"]