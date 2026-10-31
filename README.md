# zap.stream

A native [zap.stream](https://zap.stream) app built with [notedeck](https://github.com/damus-io/notedeck).

This is **ALPHA** stage software, use at your own risk!

![preview](https://nostr.download/d3ceb5b379d7faddd9d6c536292bf46e097b35fbcaf431ff0a1cbd3e3acaf052.webp)
![preview_mobile](https://nostr.download/7d303cac1b4c923e76f386b3008de5551c0a249d9ceeddafb424dfe4a34ae533.webp)


## Building

Video plays through [lumina-video](https://github.com/v0l/lumina-video) (hardware decode, zero-copy GPU frames).

- Linux: `./debian.sh` (GStreamer with VA-API for zero-copy)
- macOS / iOS: `cargo build --release` (needs `brew install ffmpeg cmake`)
- Android: `./android.sh` (cargo-ndk, JDK 21, NDK 28)

Set `ZAP_STREAM_FRAME_STATS=1` to log frame times every 5 seconds (always on for Android).
