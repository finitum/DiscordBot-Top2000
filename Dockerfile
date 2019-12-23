# -*- mode: dockerfile -*-
ARG BASE_IMAGE=rust:buster

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Cache our dependencies
WORKDIR /home/rust/
RUN USER=rust cargo new app --bin
WORKDIR /home/rust/app
COPY Cargo.* ./
RUN cargo build --release && rm ./target/release/deps/discord_bot_top2000* && rm -rf ./src/ && rm -rf .env

# Add our source code.
COPY ./src ./src
COPY ./Cargo.* ./

# Build our application.
RUN cargo build --release

# Runner
FROM debian:buster-slim
RUN apt-get update -y && apt-get install ca-certificates -y && apt-get install libopus-dev -y && apt-get install youtube-dl -y
COPY --from=builder /home/rust/app/target/release/discord_bot_top2000 /app

CMD /app
