# syntax = docker/dockerfile:1.4

###############################################################################
FROM ubuntu:20.04 AS base
###############################################################################

FROM base AS builder

ENV TZ=US/Eastern \
    DEBIAN_FRONTEND=noninteractive

# Install compile-time dependencies
RUN set -eux; \
    apt update; \
    apt install -y --no-install-recommends \
    lld clang curl ca-certificates \
    ;

# Install rustup
RUN --mount=type=cache,target=/root/.rustup \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    set -eux; \
    curl --location --fail \
    "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init" \
    --output rustup-init; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain stable; \
    rm rustup-init; 

# Add rustup to PATH & check that it works
ENV PATH=${PATH}:/root/.cargo/bin
RUN set -eux; \
  rustup --version; 

# Copy sources and build them
WORKDIR /app
COPY . .
# COPY .cargo .cargo
COPY Cargo.toml Cargo.lock ./

RUN --mount=type=cache,target=/root/.rustup \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/app/target \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections ./target/release/zero2axum ./zero2axum
ENV APP_ENVIRONMENT production
###############################################################################

FROM base AS app

SHELL ["/bin/bash", "-c"]

# Install runtime dependencies
RUN set -eux; \
    apt update; \
    apt install -y --no-install-recommends \
      ca-certificates \
      ; \
    apt clean autoclean; \
    apt autoremove --yes; \
    rm -rf /var/lib/{apt,dpkg,cache,log}/

# Copy app from builder
WORKDIR /app
COPY --from=builder /app/zero2axum .
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2axum"]