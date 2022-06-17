FROM rust:1.60-slim-bullseye as build
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config curl libssl-dev

WORKDIR /app
COPY . /app
RUN cargo build --release

# -----

FROM debian:bullseye-slim as production

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl1.1 \
    ca-certificates

RUN groupadd --gid 10001 app && \
    useradd -g app --uid 10001 --shell /usr/sbin/nologin --no-create-home --home-dir /app app

WORKDIR /app

COPY --from=build /app/target/release/pocket-proxy .
COPY --from=build /app/GeoIP2-City.mmdb ./
COPY --from=build /app/version.json ./

USER app
ENV PORT=8000
ENV HOST=0.0.0.0
EXPOSE $PORT

CMD ["/app/pocket-proxy"]
