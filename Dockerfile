FROM rust:alpine3.23 AS builder
RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Use a minimal base image for the final image
FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tinynode /app/tinynode
#RUN chmod +x /app/tinynode
CMD ["/app/tinynode"]