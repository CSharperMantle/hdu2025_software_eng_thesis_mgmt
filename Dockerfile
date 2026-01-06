FROM node:24-alpine AS frontend-builder

ARG VITE_API_BASE_URL=http://localhost:8080/api

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

WORKDIR /app

COPY frontend/ ./

RUN --mount=type=cache,id=pnpm,sharing=locked,target=/pnpm/store \
    pnpm install --frozen-lockfile

RUN echo "VITE_API_BASE_URL=$VITE_API_BASE_URL" > .env.localhost

RUN pnpm run build

FROM rust:1.92-alpine AS backend-builder

ARG DATABASE_URL
ARG SERVER_HOST=0.0.0.0
ARG SERVER_PORT=8080
ARG COOKIE_PASSPHRASE
ARG RUST_LOG=info

WORKDIR /app

RUN apk add --no-cache \
    build-base \
    perl \
    pkgconfig \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    postgresql18-dev \
    libpq-dev

# Link: https://github.com/diesel-rs/diesel/issues/3976
# Link: https://www.postgresql.org/message-id/2656597.1728582542%40sss.pgh.pa.us
RUN --mount=type=cache,id=cargo,sharing=locked,target=/usr/local/cargo/registry \
    RUSTFLAGS="-L/usr/lib/ -L/usr/lib/postgresql18/ -lpq -lpgcommon_shlib -lpgport_shlib -lm -lssl -lcrypto -ldl" \
    cargo install diesel_cli --no-default-features --features postgres

COPY backend/ ./
COPY docker/backend-cargo-static.patch ./backend-cargo-static.patch
RUN patch -p2 < ./backend-cargo-static.patch

RUN echo "DATABASE_URL=${DATABASE_URL}" > ./.env && \
    echo "SERVER_HOST=${SERVER_HOST}" >> ./.env && \
    echo "SERVER_PORT=${SERVER_PORT}" >> ./.env && \
    echo "STATIC_FILES_PATH=/app/static" >> ./.env && \
    echo "COOKIE_PASSPHRASE=${COOKIE_PASSPHRASE}" >> ./.env && \
    echo "RUST_LOG=${RUST_LOG}" >> ./.env

RUN --mount=type=cache,id=cargo,sharing=locked,target=/usr/local/cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/backend_server /usr/local/bin/backend_server

FROM alpine:3.23

ARG DATABASE_URL
ARG SERVER_HOST=0.0.0.0
ARG SERVER_PORT=8080
ARG COOKIE_PASSPHRASE
ARG RUST_LOG=info

ENV DATABASE_URL=${DATABASE_URL} \
    SERVER_HOST=${SERVER_HOST} \
    SERVER_PORT=${SERVER_PORT} \
    COOKIE_PASSPHRASE=${COOKIE_PASSPHRASE} \
    RUST_LOG=${RUST_LOG} \
    STATIC_FILES_PATH=/app/static

WORKDIR /app

RUN apk add --no-cache \
    postgresql-client \
    bash \
    curl

COPY --from=backend-builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel
COPY --from=backend-builder /usr/local/bin/backend_server /usr/local/bin/backend_server

COPY backend/migrations ./migrations
COPY backend/diesel.toml ./diesel.toml

COPY --from=frontend-builder /app/dist ./static

COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 8080

ENTRYPOINT ["/entrypoint.sh"]
