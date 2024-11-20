#!/bin/bash
apt update
apt install -y \
      build-essential \
      pkg-config \
      libavcodec-dev \
      libavformat-dev \
      libavdevice-dev \
      libavutil-dev \
      libavfilter-dev \
      libswresample-dev \
      libswscale-dev \
      libx264-dev \
      libwebp-dev \
      libssl-dev \
      libpipewire-0.3-dev \
      libpulse-dev \
      libpng-dev \
      libasound2-dev
cargo install xbuild
x build --release