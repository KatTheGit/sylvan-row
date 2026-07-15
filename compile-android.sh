# setup
rustup target add aarch64-linux-android
cargo install cargo-ndk

# exports
export ANDROID_NDK_HOME=/media/ornit/Data/android-sdk/ndk/30.0.15729638/


# compile rust into a lib.
WGPU_BACKEND=gl cargo ndk -t arm64-v8a --platform 26 -o app/src/main/jniLibs build --profile release