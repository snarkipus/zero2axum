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
    lld clang curl ca-certificates git-crypt \
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

RUN if [ -z "$GIT_CRYPT_KEY" ]; then \
        echo "$GIT_CRYPT_KEY" | base64 --decode > git_crypt_key; \
        git stash; \
        git-crypt unlock git_crypt_key; \
        rm git_crypt_key; \
    fi

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
      ca-certificates git-crypt \
      ; \
    apt clean autoclean; \
    apt autoremove --yes; \
    rm -rf /var/lib/{apt,dpkg,cache,log}/

# Copy app from builder
WORKDIR /app
COPY --from=builder /app/zero2axum .
COPY configuration configuration

RUN if [ -z "$GIT_CRYPT_KEY" ]; then \
        echo "$GIT_CRYPT_KEY" | base64 --decode > git_crypt_key; \
        git stash; \
        git-crypt unlock git_crypt_key; \
        rm git_crypt_key; \
    fi

ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2axum"]
