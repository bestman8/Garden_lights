ARG VARIANT=bookworm-slim
FROM debian:${VARIANT}
ENV DEBIAN_FRONTEND=noninteractive
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

# Arguments
ARG ESP_BOARD=all
ARG GITHUB_TOKEN

# Install dependencies
RUN apt-get update \
    && apt-get install -y pkg-config curl gcc clang libudev-dev unzip xz-utils \
    git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0 \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/* /tmp/library-scripts

# Install rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    --default-toolchain none -y --profile minimal

# Update envs
ENV PATH=${PATH}:/root/.cargo/bin

# Install extra crates
RUN ARCH=$(rustup show | grep "Default host" | sed -e 's/.* //') && \
    curl -L "https://github.com/esp-rs/espup/releases/latest/download/espup-${ARCH}" -o "/root/.cargo/bin/espup" && \
    chmod u+x "/root/.cargo/bin/espup" && \
    curl -L "https://github.com/esp-rs/espflash/releases/latest/download/cargo-espflash-${ARCH}.zip" -o "/root/.cargo/bin/cargo-espflash.zip" && \
    unzip "/root/.cargo/bin/cargo-espflash.zip" -d "/root/.cargo/bin/" && \
    rm "/root/.cargo/bin/cargo-espflash.zip" && \
    chmod u+x "/root/.cargo/bin/cargo-espflash" && \
    curl -L "https://github.com/esp-rs/espflash/releases/latest/download/espflash-${ARCH}.zip" -o "/root/.cargo/bin/espflash.zip" && \
    unzip "/root/.cargo/bin/espflash.zip" -d "/root/.cargo/bin/" && \
    rm "/root/.cargo/bin/espflash.zip" && \
    chmod u+x "/root/.cargo/bin/espflash" && \
    curl -L "https://github.com/esp-rs/embuild/releases/latest/download/ldproxy-${ARCH}.zip" -o "/root/.cargo/bin/ldproxy.zip" && \
    unzip "/root/.cargo/bin/ldproxy.zip" -d "/root/.cargo/bin/" && \
    rm "/root/.cargo/bin/ldproxy.zip" && \
    chmod u+x "/root/.cargo/bin/ldproxy" && \
    curl -L "https://github.com/esp-rs/esp-web-flash-server/releases/latest/download/web-flash-${ARCH}.zip" -o "/root/.cargo/bin/web-flash.zip" && \
    unzip "/root/.cargo/bin/web-flash.zip" -d "/root/.cargo/bin/" && \
    rm "/root/.cargo/bin/web-flash.zip" && \
    chmod u+x "/root/.cargo/bin/web-flash"

# Install Xtensa Rust
RUN if [ -n "${GITHUB_TOKEN}" ]; then export GITHUB_TOKEN=${GITHUB_TOKEN}; fi  \
    && /root/.cargo/bin/espup install \
    --targets "${ESP_BOARD}" \
    --log-level debug \
    --export-file /root/export-esp.sh

# Set default toolchain
RUN rustup default esp

# Activate ESP environment
RUN echo "source /root/export-esp.sh" >> /root/.bashrc

CMD [ "/bin/bash" ]
