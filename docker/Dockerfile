# syntax = docker/dockerfile:1

# Stage 1: Build Wasm UI and Rust Server
FROM rust:1.94.0-slim-bullseye AS builder
RUN apt-get update && apt-get install -y curl pkg-config libssl-dev build-essential
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY wordle-engine ./wordle-engine
COPY wordle-ui ./wordle-ui
COPY wordle-server ./wordle-server

WORKDIR /app/wordle-ui
RUN trunk build --release

WORKDIR /app/wordle-server
RUN cargo build --release

# Stage 2: Production Server
FROM debian:bullseye-slim AS prod

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN groupadd -g 1001 reactle && \
    useradd -r -u 1001 -g reactle reactle && \
    mkdir -p /app/dist /app/data && \
    chown -R reactle:reactle /app

VOLUME /app/data

COPY --from=builder --chown=reactle:reactle /app/wordle-ui/dist /app/dist
COPY --from=builder --chown=reactle:reactle /app/target/release/wordle-server /app/wordle-server-bin

USER reactle
EXPOSE 7583

CMD ["/app/wordle-server-bin"]
