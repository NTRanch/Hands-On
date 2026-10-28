#!/usr/bin/env bash
# Hands-On — Gradle-free APK packaging (debug-signed).
#
# Pipeline: cargo-ndk builds libhands_on_app.so -> aapt2 links the manifest
# (+ a generated launcher icon) against android.jar -> the .so is added under
# lib/arm64-v8a/ stored (uncompressed, page-aligned via zipalign -p) ->
# apksigner signs with a DEBUG keystore. Debug signing is for sideload/F-Droid
# recipe development only; a release build must use a real key.
#
# Required env: ANDROID_SDK_ROOT (with platforms/android-34,
# build-tools/34.0.0, ndk/26.1.10909125), JAVA_HOME (keytool), cargo-ndk on
# PATH. Run from the repository root.
set -euo pipefail

SDK="${ANDROID_SDK_ROOT:?set ANDROID_SDK_ROOT}"
BT="$SDK/build-tools/34.0.0"
JAR="$SDK/platforms/android-34/android.jar"
OUT="hands-on-app/packaging/out"
KEYSTORE="hands-on-app/packaging/debug.keystore"

export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$SDK/ndk/26.1.10909125}"

echo "== cargo-ndk build (arm64-v8a, release) =="
cargo ndk -t arm64-v8a build --release -p hands-on-app

mkdir -p "$OUT/lib/arm64-v8a" "$OUT/res/mipmap-hdpi"
cp target/aarch64-linux-android/release/libhands_on_app.so "$OUT/lib/arm64-v8a/"

# oboe (rodio/cpal) links the NDK's shared libc++ on Android
# (cpal-shared-stdcxx); the runtime library ships in the APK.
CXXLIB="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so"
if [ -f "$CXXLIB" ]; then
    cp "$CXXLIB" "$OUT/lib/arm64-v8a/"
    echo "bundled libc++_shared.so (oboe runtime)"
fi

echo "== launcher icon (generated placeholder art) =="
python3 - "$OUT/res/mipmap-hdpi/ic_launcher.png" <<'PY'
import struct, sys, zlib

w = h = 72
rows = []
for y in range(h):
    row = bytearray([0])
    for x in range(w):
        # teal disc on near-black — matches the app's accent color
        dx, dy = x - w / 2, y - h / 2
        inside = dx * dx + dy * dy < (w * 0.36) ** 2
        row += bytes((25, 227, 194, 255) if inside else (10, 10, 14, 255))
    rows.append(bytes(row))
raw = b"".join(rows)


def chunk(tag, data):
    c = struct.pack(">I", len(data)) + tag + data
    return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)


png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(raw))
    + chunk(b"IEND", b"")
)
open(sys.argv[1], "wb").write(png)
PY

echo "== aapt2 compile + link =="
"$BT/aapt2" compile --dir "$OUT/res" -o "$OUT/res.zip"
"$BT/aapt2" link \
    -I "$JAR" \
    --manifest hands-on-app/packaging/AndroidManifest.xml \
    "$OUT/res.zip" \
    --auto-add-overlay \
    -o "$OUT/unsigned.apk"

echo "== add native libs (stored) + align =="
(cd "$OUT" && zip -q -0 unsigned.apk lib/arm64-v8a/libhands_on_app.so)
if [ -f "$OUT/lib/arm64-v8a/libc++_shared.so" ]; then
    (cd "$OUT" && zip -q -0 unsigned.apk lib/arm64-v8a/libc++_shared.so)
fi
"$BT/zipalign" -f -p 4 "$OUT/unsigned.apk" "$OUT/aligned.apk"

echo "== debug keystore + sign =="
if [ ! -f "$KEYSTORE" ]; then
    "$JAVA_HOME/bin/keytool" -genkeypair -v \
        -keystore "$KEYSTORE" -storepass android -keypass android \
        -alias androiddebugkey -keyalg RSA -keysize 2048 -validity 10000 \
        -dname "CN=Android Debug,O=Android,C=US"
fi
"$BT/apksigner" sign \
    --ks "$KEYSTORE" --ks-pass pass:android --key-pass pass:android \
    --out "$OUT/hands-on-debug.apk" "$OUT/aligned.apk"

echo "== verify =="
"$BT/apksigner" verify --print-certs "$OUT/hands-on-debug.apk"
"$BT/aapt2" dump badging "$OUT/hands-on-debug.apk" | head -5
ls -l "$OUT/hands-on-debug.apk"
sha256sum "$OUT/hands-on-debug.apk"
echo "APK-DONE"
