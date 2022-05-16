FROM rust:1.60 as builder
# The exact rust version comes from the toolchain file.
WORKDIR /opt/swc
COPY . .
RUN cargo install --profile production --path web

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/swc-web /usr/local/bin/swc-web

EXPOSE 443
EXPOSE 80
CMD ["swc-web"]
