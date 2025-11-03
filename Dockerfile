FROM alpine:edge as build

WORKDIR /work
RUN    apk --no-cache add rust cargo g++ clang jq linux-headers protoc clang-dev perl build-base

ENV PKG_CONFIG_ALLOW_CROSS=true \
    PKG_CONFIG_ALL_STATIC=true \
    RUSTFLAGS="-C target-feature=+crt-static"

COPY . .

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/work/target \
    cargo build --release --target riscv64-alpine-linux-musl -p dora-cli && \
        cp target/riscv64-alpine-linux-musl/release/dora /root/dora

FROM scratch as export
WORKDIR /
COPY --from=build /root/dora /dora

ENTRYPOINT ["/dora"]
