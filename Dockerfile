FROM rust:1.70 as builder
# The exact rust version comes from the toolchain file.
WORKDIR /opt/swc
COPY . .
RUN cargo install --profile production --path web &&\
	cargo install --profile production --path cli

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y git libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/subweight-web /usr/local/bin/subweight-web
COPY --from=builder /usr/local/cargo/bin/subweight /usr/local/bin/subweight
COPY web/static web/static

EXPOSE 443
EXPOSE 80
CMD ["swc-web"]
