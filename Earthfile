# This file is used to allow MacOS users to build images with DevEnv
# (https://github.com/cachix/devenv/issues/430)
VERSION 0.8
FROM scratch

backend:
    FROM rust:latest
    COPY --dir assets migration src templates .
    COPY Cargo.toml Cargo.lock .
    RUN cargo build --release
    SAVE ARTIFACT target/release/bookclub-cli

build:
    FROM debian:stable-slim
    COPY --dir config .
    COPY +backend/bookclub-cli .
    ENTRYPOINT ["./bookclub-cli"]
    ARG --required EARTHLY_GIT_HASH
    ARG --required EARTHLY_GIT_BRANCH
    SAVE IMAGE --push \
        thoward27/bookclub:$EARTHLY_GIT_HASH \
        thoward27/bookclub:$EARTHLY_GIT_BRANCH

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
