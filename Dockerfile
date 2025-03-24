FROM rust:1.85.1-alpine3.21 AS builder
USER root
WORKDIR /src
COPY . /src
RUN cargo build --release

FROM scratch
COPY --from=builder /src/target/release/version-bump /app/
CMD ["/app/version-bump"]