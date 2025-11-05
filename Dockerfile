# syntax=docker/dockerfile:1.6

########################################
# Build stage: full Rust toolchain
########################################
FROM rust:1.77-bullseye AS builder

WORKDIR /app

# Install native build dependencies commonly required by Rust crates.
RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        pkg-config \
        libssl-dev \
        libclang-dev \
        clang \
    && rm -rf /var/lib/apt/lists/*

# Leverage Docker layer caching: copy manifests first.
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY fuzz fuzz

# Build the release binary.
RUN cargo build --release

########################################
# Runtime stage: minimal image
########################################
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged user.
RUN useradd --system --user-group restful

# Copy binary from builder stage.
COPY --from=builder /app/target/release/restful-rust /usr/local/bin/restful-rust

USER restful
EXPOSE 8080
ENV RUST_LOG=restful_rust=info

ENTRYPOINT ["/usr/local/bin/restful-rust"]
