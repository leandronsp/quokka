FROM rust AS base
WORKDIR /app

FROM base AS build
COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --release

FROM debian:stable-slim AS prod
COPY --from=build /app/target/release/quokka /usr/bin/quokka
EXPOSE 3000
CMD ["quokka"]
