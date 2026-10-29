FROM mcr.microsoft.com/devcontainers/base:jammy

# mediasoup compiles its worker when a downloaded binary cannot run on this image.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        python3 \
        python3-pip \
        python3-venv \
    && rm -rf /var/lib/apt/lists/*
