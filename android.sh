#!/bin/bash

git clone https://github.com/v0l/ffmpeg-kit.git
export ANDROID_SDK_ROOT=$ANDROID_HOME
#cd ffmpeg-kit && ./android.sh \
#  --disable-x86 \
#  --disable-x86-64 \
#  --disable-arm-v7a \
#  --disable-arm-v7a-neon \
#  --enable-openssl \
#  --api-level=28 \
#  --no-ffmpeg-kit-protocols \
#  --no-archive

NDK_VER="28.0.12433566"
ARCH="arm64"
PLATFORM="android"
TRIPLET="aarch64-linux-android"
export FFMPEG_DIR="$(pwd)/ffmpeg-kit/prebuilt/$PLATFORM-$ARCH/ffmpeg"
export PKG_CONFIG_SYSROOT_DIR="$(pwd)/ffmpeg-kit/prebuilt/$PLATFORM-$ARCH/pkgconfig"

# DIRTY HACK !!
mkdir -p ./target/x/debug/android/$ARCH/cargo/$TRIPLET/release/deps
mkdir -p ./target/x/release/android/$ARCH/cargo/$TRIPLET/release/deps
cp "$ANDROID_HOME/ndk/$NDK_VER/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/$TRIPLET/35/libcamera2ndk.so" \
  ./target/x/debug/android/$ARCH/cargo/$TRIPLET/debug/deps
cp "$ANDROID_HOME/ndk/$NDK_VER/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/$TRIPLET/35/libcamera2ndk.so" \
  ./target/x/release/android/$ARCH/cargo/$TRIPLET/release/deps

x build --arch $ARCH --platform $PLATFORM --verbose --release
