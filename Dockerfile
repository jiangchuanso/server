FROM rust:bookworm AS builder

ENV DEBIAN_FRONTEND=noninteractive

RUN wget -qO- https://apt.repos.intel.com/intel-gpg-keys/GPG-PUB-KEY-INTEL-SW-PRODUCTS.PUB | gpg --dearmor -o /usr/share/keyrings/oneapi-archive-keyring.gpg && \
    echo "deb [signed-by=/usr/share/keyrings/oneapi-archive-keyring.gpg] https://apt.repos.intel.com/oneapi all main" > /etc/apt/sources.list.d/oneAPI.list

RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    liblapack-dev \
    libblas-dev \
    intel-oneapi-mkl \
    intel-oneapi-mkl-devel

ENV MKLROOT=/opt/intel/oneapi/mkl/latest

WORKDIR /app
COPY . .

RUN cargo build -p server --release -vv

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /opt/intel/oneapi/compiler/latest/lib/libiomp5.so /usr/lib/x86_64-linux-gnu/libiomp5.so

ENV MODELS_DIR=/app/models
ENV NUM_WORKERS=1
ENV IP=0.0.0.0
ENV PORT=3000
# ENV ENV_API_KEY=
ENV RUST_LOG=info

EXPOSE 3000

ENTRYPOINT ["/app/server"]
