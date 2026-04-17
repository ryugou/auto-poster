# --- dev ---
FROM rust:1.87.0-bookworm AS dev
RUN cargo install cargo-watch sqlx-cli --locked
WORKDIR /app

# --- build ---
FROM rust:1.87.0-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release

# --- release ---
FROM debian:bookworm-slim AS release
RUN apt-get update \
    && apt-get install -y --no-install-recommends libsqlite3-0 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/auto-poster /usr/local/bin/
COPY --from=build /app/config /etc/auto-poster/config
RUN mkdir -p /var/lib/auto-poster
ENV APP_DATABASE_URL="sqlite:/var/lib/auto-poster/auto-poster.db"
WORKDIR /etc/auto-poster
ENTRYPOINT ["auto-poster", "--config-dir", "/etc/auto-poster/config"]
CMD ["--help"]
