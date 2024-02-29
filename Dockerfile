FROM rust:1.75.0 as base

WORKDIR /app

FROM base AS builder
RUN mkdir /temp
COPY Cargo.toml /temp
COPY Cargo.lock /temp
COPY src /temp/src

RUN cd /temp && cargo build --release


FROM base as release

COPY --from=builder temp/target/release/rinha-de-backend-2024-rust rinha-de-backend-2024-rust

EXPOSE 3000

CMD ["./rinha-de-backend-2024-rust"]
