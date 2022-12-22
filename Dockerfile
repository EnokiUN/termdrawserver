# syntax=docker/dockerfile:1
FROM rust:slim-buster as builder

WORKDIR /termdrawserver

COPY Cargo.lock Cargo.toml ./
COPY ./src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/termdrawserver/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/termdrawserver/target \
    cp ./target/release/termdrawserver /usr/local/bin/termdrawserver

FROM debian:buster-slim

# Don't forget to also publish these ports in the docker-compose.yml file.
ARG PORT=8182

EXPOSE $PORT
ENV SERVER_ADDRESS 0.0.0.0
ENV SERVER_PORT $PORT

ENV RUST_LOG debug

COPY --from=builder /usr/local/bin/termdrawserver /bin/termdrawserver

CMD ["/bin/termdrawserver"]
