# syntax = docker/dockerfile:1

# Stage 1: Build Rust App
FROM rust:1.94.0-slim-bullseye AS builder
RUN apt-get update && apt-get install -y curl pkg-config libssl-dev build-essential
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

WORKDIR /app
COPY Cargo.toml ./
COPY wordle-engine ./wordle-engine
COPY wordle-ui ./wordle-ui

WORKDIR /app/wordle-ui
RUN trunk build --release

## Stage 2: Production image
FROM nginx:1.24.0-alpine AS prod

# Clean defaults
RUN rm -rf /etc/nginx/conf.d/* /usr/share/nginx/html/*

# Use our single hardened config
COPY docker/etc/nginx/nginx.conf /etc/nginx/nginx.conf

# Copy build output
COPY --from=builder /app/wordle-ui/dist /usr/share/nginx/html

# Set up non-root user
RUN addgroup -g 1001 -S reactle && \
    adduser -S -D -H -u 1001 -G reactle reactle && \
    chown -R reactle:reactle /usr/share/nginx/html /var/cache/nginx /var/log/nginx /etc/nginx /tmp

USER reactle
EXPOSE 7583

CMD ["nginx", "-g", "daemon off;"]
