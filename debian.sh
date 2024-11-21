#!/bin/bash
apt update && \
    apt install -y \
    build-essential \
    libx264-dev \
    libx265-dev \
    libwebp-dev \
    libvpx-dev \
    libopus-dev \
    libdav1d-dev \
    libpipewire-0.3-dev \
    libasound2-dev \
    nasm \
    libclang-dev \
    squashfs-tools
git clone --single-branch --branch release/7.1 https://git.v0l.io/ffmpeg/FFmpeg.git && \
    cd FFmpeg && \
    ./configure \
    --prefix=${FFMPEG_DIR} \
    --disable-programs \
    --disable-doc \
    --disable-network \
    --enable-gpl \
    --enable-libx264 \
    --enable-libx265 \
    --enable-libwebp \
    --enable-libvpx \
    --enable-libopus \
    --enable-libdav1d \
    --disable-static \
    --disable-postproc \
    --enable-shared && \
    make -j$(nproc) install

cargo install xbuild
x build --release --format appimage