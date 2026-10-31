#!/bin/bash
set -e
apt update
apt install -y \
    build-essential \
    cmake \
    pkg-config \
    libclang-dev \
    libasound2-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libx11-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-libav
cargo build --release
