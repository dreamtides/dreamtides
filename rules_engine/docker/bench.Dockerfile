FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    rsync \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain stable \
    && rustup component add rustfmt clippy \
    && cargo --version \
    && rustc --version

RUN useradd -ms /bin/bash runner \
    && mkdir -p /workspace \
    && chown -R runner:runner /workspace /usr/local/cargo /usr/local/rustup

USER runner
WORKDIR /workspace

CMD ["bash", "-lc", "sleep infinity"]


