FROM cgr.dev/chainguard/rust:latest AS builder
WORKDIR /app
COPY src ./src
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

FROM cgr.dev/chainguard/glibc-dynamic:latest AS runner
COPY --from=builder --chown=nonroot:nonroot \
    /app/target/release/kubernetes-events /usr/local/bin/kubernetes-events
CMD ["/usr/local/bin/kubernetes-events"]
