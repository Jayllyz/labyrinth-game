FROM rust:1-slim AS base
WORKDIR /app
COPY . .
RUN cargo build -p client --release --bin client

FROM debian:stable-slim AS prod
WORKDIR /app

RUN apt-get update && apt-get install -y \
    xvfb \
    libx11-6 \
    libxcursor1 \
    libxi6 \
    libvulkan1 \
    libegl1 \
    mesa-vulkan-drivers \
    libgl1-mesa-dri \
    libgl1-mesa-glx \
    mesa-utils \
    && rm -rf /var/lib/apt/lists/*

COPY --from=base /app/target/release/client /usr/local/bin
COPY e2e/server /usr/local/bin
COPY e2e/start.sh /app/start.sh

RUN chmod +x /usr/local/bin/client /usr/local/bin/server /app/start.sh

ENV WINIT_UNIX_BACKEND=x11
ENV DISPLAY=:99
ENV XDG_RUNTIME_DIR=/tmp/runtime-dir
ENV WGPU_BACKEND=vulkan
ENV LIBGL_ALWAYS_SOFTWARE=1
ENV __GLX_VENDOR_LIBRARY_NAME=mesa
ENV WGPU_ADAPTER_NAME=llvmpipe
ENV WGPU_PREFERRED_FORMAT=Rgba8Unorm

ENTRYPOINT ["/app/start.sh"]
