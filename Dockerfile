# syntax=docker/dockerfile:1

FROM rust:1.98-bookworm AS build
WORKDIR /source
# Source binds keep adopter files out of image layers; cache mounts retain Cargo work across builds.
RUN --mount=type=bind,source=Cargo.toml,target=/source/Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=/source/Cargo.lock \
    --mount=type=bind,source=crates,target=/source/crates \
    --mount=type=cache,target=/source/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    CARGO_INCREMENTAL=0 cargo build --release --locked -p aep-service \
    && cp target/release/aep-service /usr/local/bin/aep-service \
    && aep_revision="$(sed -n 's/.*github.com\/beyond10x\/aep?tag=[^#]*#\([0-9a-f]*\)"/\1/p' Cargo.lock | head -n 1)" \
    && test -n "$aep_revision" \
    && aep_checkout="$(find /usr/local/cargo/git/checkouts -mindepth 2 -maxdepth 2 -type d -name "$(printf '%s' "$aep_revision" | cut -c 1-7)" -print -quit)" \
    && test -n "$aep_checkout" \
    && install -d /usr/local/share/aep-service/definitions/artifacts \
    && cp -R "$aep_checkout/protocols" /usr/local/share/aep-service/definitions/protocols \
    && cp -R "$aep_checkout/principles" /usr/local/share/aep-service/definitions/principles \
    && cp -R "$aep_checkout/workflows" /usr/local/share/aep-service/definitions/workflows \
    && cp -R "$aep_checkout/profiles" /usr/local/share/aep-service/definitions/profiles \
    && cp -R "$aep_checkout/artifacts/lifecycles" /usr/local/share/aep-service/definitions/artifacts/lifecycles \
    && cp -R "$aep_checkout/drivers" /usr/local/share/aep-service/definitions/drivers

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 aep
COPY --from=build /usr/local/bin/aep-service /usr/local/bin/aep-service
COPY --from=build /usr/local/share/aep-service/definitions /definitions
USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/aep-service"]
