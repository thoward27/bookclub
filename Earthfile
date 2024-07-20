# This file is used to allow MacOS users to build images with DevEnv
# (https://github.com/cachix/devenv/issues/430)
VERSION 0.8
FROM scratch

frontend:
    FROM node:latest
    RUN npm install -g pnpm
    COPY --dir frontend .
    WORKDIR frontend
    RUN pnpm install
    RUN pnpm run build
    SAVE ARTIFACT dist

backend:
    FROM rust:latest
    COPY --dir assets migration src .
    COPY Cargo.toml Cargo.lock .
    RUN cargo build --release
    SAVE ARTIFACT target/release/bookclub-cli

prod:
    FROM debian:stable-slim
    COPY --dir config .
    COPY +frontend/dist /frontend/dist
    COPY +backend/bookclub-cli .
    ENTRYPOINT ["./bookclub-cli"]
    SAVE IMAGE --push thoward27/bookclub:latest 
