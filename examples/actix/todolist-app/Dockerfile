FROM rust:1.72 as builder

WORKDIR /usr/app
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /usr/app/example

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    userland

COPY ./Cargo.toml ./Cargo.toml
RUN mkdir src && echo "fn main(){}" > ./src/main.rs

RUN cargo build --release
COPY ./src ./src

# 5. Build for release.
RUN cargo build --target x86_64-unknown-linux-musl --release
#--------
FROM alpine
HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 CMD [ "curl --fail http://localhost:8080/health" ]
RUN apk add curl
RUN apk add libc6-compat

WORKDIR /opt
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /usr/app/example/target/x86_64-unknown-linux-musl/release/example ./

RUN chown userland:userland ./example

USER userland

CMD ["/opt/example"]
