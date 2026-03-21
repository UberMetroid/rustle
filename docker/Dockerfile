# syntax = docker/dockerfile:1

# Stage 1: Build Rust App (No Node.js needed!)
FROM rust:1.94.0-slim-bullseye AS builder
RUN apt-get update && apt-get install -y curl pkg-config libssl-dev build-essential
# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown
# Install trunk
RUN cargo install trunk

WORKDIR /app
COPY Cargo.toml ./
COPY wordle-engine ./wordle-engine
COPY wordle-ui ./wordle-ui

WORKDIR /app/wordle-ui
RUN trunk build --release

## Stage 2: Production image
FROM nginx:1.24.0-alpine AS prod
COPY docker/etc/nginx/nginx.conf /etc/nginx/nginx.conf
COPY docker/etc/nginx/conf.d/default.conf /etc/nginx/conf.d/default.conf
# Copy Trunk build output to Nginx html folder
COPY --from=builder /app/wordle-ui/dist /usr/share/nginx/html
COPY docker/build_system.sh .
RUN ./build_system.sh && rm ./build_system.sh

EXPOSE 7583
USER reactle
