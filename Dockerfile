FROM lukemathwalker/cargo-chef:0.1.68-rust-1.84.0-bookworm AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin todo_app

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends openssl ca-certificates && \
    apt-get autoremove -y && \
    apt-get clean -y && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/todo_app todo_app
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static

EXPOSE 8080
ENTRYPOINT ["./todo_app"]
