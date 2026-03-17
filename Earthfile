# This file is used to allow MacOS users to build images with DevEnv
# (https://github.com/cachix/devenv/issues/430)
VERSION 0.8
FROM scratch

deps:
    FROM rust:1.9.4-trixie
    RUN cargo install cargo-chef
    WORKDIR /app

recipe:
    FROM +deps
    COPY --dir assets migration src templates .
    COPY Cargo.toml Cargo.lock .
    RUN cargo chef prepare --recipe-path recipe.json
    SAVE ARTIFACT recipe.json

backend:
    FROM +deps
    COPY +recipe/recipe.json recipe.json
    # Build dependencies - this layer is cached unless Cargo.toml/Cargo.lock change
    RUN cargo chef cook --release --recipe-path recipe.json
    # Now copy source and build - only your code compiles
    COPY --dir assets migration src templates .
    COPY Cargo.toml Cargo.lock .
    ARG --required EARTHLY_GIT_HASH
    RUN VERSION=$EARTHLY_GIT_HASH cargo build --release
    COPY config/production.yaml config/production.yaml
    SAVE ARTIFACT target/release/bookclub-cli

build:
    FROM debian:stable-slim
    RUN apt update -yy && apt install -yy openssl
    WORKDIR /usr/local/src
    COPY --dir config assets .
    COPY +backend/bookclub-cli .
    ENTRYPOINT ["./bookclub-cli"]
    ARG --required EARTHLY_GIT_HASH
    ARG --required EARTHLY_GIT_BRANCH
    SAVE IMAGE --push \
        thoward27/bookclub:$EARTHLY_GIT_HASH \
        thoward27/bookclub:$EARTHLY_GIT_BRANCH

build-all-platforms:
    BUILD --platform=linux/amd64 --platform=linux/arm64 +build

levant-render:
    FROM hashicorp/levant
    WORKDIR /usr/local/src
    COPY --dir levant/ ./
    COPY Nomadfile ./
    FOR vars IN $(ls ./levant)
        RUN levant render -var-file=./levant/$vars ./Nomadfile > ./Nomadfile.$vars
    END
    SAVE ARTIFACT --force Nomadfile.*

nomad-validate:
    FROM hashicorp/nomad
    WORKDIR /usr/local/src
    COPY +levant-render/Nomadfile.* ./
    FOR nomadfile IN $(ls ./Nomadfile.*)
        RUN nomad validate $nomadfile
    END
