FROM rust AS base
WORKDIR /app

FROM base AS prod
COPY . .
RUN cargo build --release
EXPOSE 3000
CMD ["cargo", "run", "--release"]
