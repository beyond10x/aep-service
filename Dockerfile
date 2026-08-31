FROM rust:1.85-bookworm AS build
WORKDIR /source
COPY . .
RUN cargo build --release --locked -p aep-service

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 aep
COPY --from=build /source/target/release/aep-service /usr/local/bin/aep-service
USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/aep-service"]
