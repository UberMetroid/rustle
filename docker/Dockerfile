# syntax = docker/dockerfile:1

# Stage 1: Build Rust Wasm
FROM rust:1.94.0-slim-bullseye AS rust_builder
RUN apt-get update && apt-get install -y curl pkg-config libssl-dev build-essential
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
WORKDIR /rust-app
COPY wordle-engine ./wordle-engine
WORKDIR /rust-app/wordle-engine
RUN wasm-pack build --target web

# Stage 2: Build Node App
FROM node:18.12.1-alpine3.15 AS node_modules
WORKDIR /app
COPY package-lock.json package.json ./
RUN npm install
COPY . .
# Copy Wasm package from rust_builder
COPY --from=rust_builder /rust-app/wordle-engine/pkg ./src/wordle-engine-pkg
# Disable ESLint for the generated glue code
RUN sed -i '1i/* eslint-disable */' ./src/wordle-engine-pkg/wordle_engine.js

FROM node_modules AS prod_builder
RUN npm run build

## Stage 3: Production image
FROM nginx:1.24.0-alpine AS prod
COPY docker/etc/nginx/nginx.conf /etc/nginx/nginx.conf
COPY docker/etc/nginx/conf.d/default.conf /etc/nginx/conf.d/default.conf
RUN mkdir /usr/share/nginx/html/react-wordle
COPY --from=prod_builder /app/build /usr/share/nginx/html/react-wordle
COPY docker/build_system.sh .
RUN ./build_system.sh && rm ./build_system.sh
# port use by Nginx within docker network.
EXPOSE 7583
USER reactle
