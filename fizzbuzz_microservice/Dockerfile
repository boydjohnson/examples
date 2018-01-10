

FROM rust:latest

RUN mkdir -p build /build/bin

WORKDIR build

COPY workload .

RUN cargo build --release && \
    cp target/release/workload /build/bin/workload

ENV PATH $PATH:/build/bin
