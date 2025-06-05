ARG RUST_VERSION=1.87
FROM rust:${RUST_VERSION} AS build
WORKDIR /build
RUN apt-get install git curl
ENV DATABASE_URL="sqlite://./db/db.db"
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=askama.toml,target=askama.toml \
    --mount=type=bind,source=assets,target=assets \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=bind,source=templates,target=templates \
    --mount=type=bind,source=db,target=db \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && \
    cp target/release/recipe-server /bin/rs
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    appuser
USER appuser
WORKDIR /home/appuser
COPY --chown=appuser:appuser assets ./assets
COPY --chown=appuser:appuser migrations ./migrations
COPY --chown=appuser:appuser db ./db
COPY --chown=appuser:appuser secrets ./secrets
COPY --chown=appuser:appuser templates ./templates
EXPOSE 3000
CMD ["/bin/rs", "--ip", "0.0.0.0", "--db-uri", "sqlite://db/db.db"]
