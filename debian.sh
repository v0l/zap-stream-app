#!/bin/bash
apt update && apt install -y \
      build-essential \
      pkg-config \
      libx264-dev \
      libwebp-dev \
      libssl-dev \
      libpipewire-0.3-dev \
      libpulse-dev \
      libpng-dev \
      libasound2-dev \
      libclang-dev \
      nasm
git clone --single-branch --branch release/7.1 https://git.v0l.io/ffmpeg/FFmpeg.git && \
    cd FFmpeg && \
    ./configure \
    --disable-programs \
    --disable-doc \
    --enable-gpl \
    --enable-libx264 \
    --enable-libwebp \
    --disable-postproc \
    --enable-static \
    --disable-shared && \
    make -j$(nproc) install
export CARGO_FEATURE_STATIC=1
cargo install xbuild
x build --release