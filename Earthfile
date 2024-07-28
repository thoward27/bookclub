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

build:
    FROM debian:stable-slim
    COPY --dir config .
    COPY +frontend/dist /frontend/dist
    COPY +backend/bookclub-cli .
    ENTRYPOINT ["./bookclub-cli"]
    ARG --required GIT_BRANCH
    ARG --required GIT_COMMIT
    ARG --required TAG=latest
    SAVE IMAGE --push \
        thoward27/bookclub:$TAG \
        thoward27/bookclub:$GIT_BRANCH \
        thoward27/bookclub:$GIT_COMMIT
