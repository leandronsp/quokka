FROM rust AS base
WORKDIR /app

FROM base AS prod
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --release
COPY . .
EXPOSE 3000
CMD ["cargo", "run", "--release"]
