#!/bin/bash
set -e

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/28.2.13676358}"
export ANDROID_NDK="$ANDROID_NDK_HOME"
export CMAKE_ANDROID_NDK="$ANDROID_NDK_HOME"

cargo ndk --platform 29 -t arm64-v8a -o android/app/src/main/jniLibs build --release --lib

echo "sdk.dir=$ANDROID_HOME" > android/local.properties
cd android
./gradlew :app:assembleRelease
