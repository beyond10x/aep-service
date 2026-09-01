# syntax=docker/dockerfile:1

FROM rust:1.85-bookworm AS build
WORKDIR /source
# Source binds keep adopter files out of image layers; cache mounts retain Cargo work across builds.
RUN --mount=type=bind,source=Cargo.toml,target=/source/Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=/source/Cargo.lock \
    --mount=type=bind,source=crates,target=/source/crates \
    --mount=type=cache,target=/source/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    CARGO_INCREMENTAL=0 cargo build --release --locked -p aep-service \
    && cp target/release/aep-service /usr/local/bin/aep-service

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 aep
COPY --from=build /usr/local/bin/aep-service /usr/local/bin/aep-service
USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/aep-service"]
