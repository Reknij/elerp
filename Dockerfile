# Rust
FROM rust:latest AS builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
WORKDIR /server
COPY ./server .
RUN cargo build --target x86_64-unknown-linux-musl --release

# Node
FROM node:20-alpine as node_build
WORKDIR /web
COPY ./web .
RUN npm install
RUN npm run build

# Alpine
FROM alpine:3.17
COPY --from=builder /server/target/x86_64-unknown-linux-musl/release/elerp /
COPY --from=node_build /web/dist/ /dist
VOLUME ["/data"]
CMD /elerp --data-path /data