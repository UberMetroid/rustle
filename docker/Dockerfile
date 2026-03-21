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

# Remove default config that tries to use port 80
RUN rm /etc/nginx/conf.d/default.conf

COPY docker/etc/nginx/nginx.conf /etc/nginx/nginx.conf
COPY docker/etc/nginx/conf.d/default.conf /etc/nginx/conf.d/default.conf

# Copy build output
COPY --from=builder /app/wordle-ui/dist /usr/share/nginx/html
COPY docker/build_system.sh .
RUN ./build_system.sh && rm ./build_system.sh

# Run Nginx on a non-privileged port
EXPOSE 7583
USER reactle

CMD ["nginx", "-g", "daemon off;"]
