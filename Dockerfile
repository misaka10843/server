FROM rustlang/rust:nightly@sha256:2d1d69e289f49b809e90bde331469bb95eaf72d9f10aee1f9b7b951a6edc414c AS builder

#RUN echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm main contrib non-free" > /etc/apt/sources.list && \
#    echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm-updates main contrib non-free" >> /etc/apt/sources.list && \
#    echo "deb https://mirrors.tuna.tsinghua.edu.cn/debian-security bookworm-security main contrib non-free" >> /etc/apt/sources.list


RUN apt-get update && \
    apt-get install -y \
    clang \
    lld \
    libssl-dev \
    pkg-config \
    git\
    libgit2-dev\
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN RUSTUP_PERMIT_COPY_RENAME=1 cargo build --release

FROM debian:bookworm-slim@sha256:90522eeb7e5923ee2b871c639059537b30521272f10ca86fdbbbb2b75a8c40cd

COPY --from=builder /app/target/release/thcdb_rs /usr/local/bin/thcdb_rs

CMD ["thcdb_rs"]
