# `cargo-chef` is a cargo-subcommand that provides
# enhanced Docker layer caching for Rust projects.
FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app
# Force `rustup` to sync the toolchain in the base `chef` layer
# so that it doesn't happen more than once
COPY rust-toolchain.toml .
RUN rustup show active-toolchain

FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project's dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Build our project
# TODO - make Dockerfile a template and pass it the app name
RUN cargo build --release --bin my-app-web

FROM debian:bookworm-slim AS runtime
WORKDIR /app
# TODO pass the app name instead once Dockerfile is a template
COPY --from=builder /app/target/release/my-app-web bin
# COPY server/config config

# Enable backtraces to simplify debugging
# production panics.
ENV RUST_BACKTRACE=1
# We don't want `anyhow` to capture backtraces for
# "routine" errors. Just panics.
ENV RUST_LIB_BACKTRACE=0
ENV APP_ENVIRONMENT=production
ENV APP_SERVER__PORT=3000
ENV APP_SERVER__IP="0.0.0.0"
ENTRYPOINT ["./bin"]
EXPOSE 3000
