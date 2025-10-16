FROM rust:1.90 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ogis .
COPY fonts ./fonts
COPY fonts.yaml .

EXPOSE 3000

CMD ["./ogis"]