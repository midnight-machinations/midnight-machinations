FROM mcr.microsoft.com/devcontainers/base:jammy

# C toolchain for Rust crates with build scripts.
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*
