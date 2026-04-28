# setup
rustup target add aarch64-linux-android
cargo install cargo-ndk

# exports
ANDROID_SDK_ROOT=~/Android/Sdk/
ANDROID_NDK_ROOT=~/Android/Sdk/ndk/29.0.14206865/


# compile rust into a lib.
WGPU_BACKEND=gl cargo ndk -t arm64-v8a -o android/app/src/main/jniLibs build --profile release 
mv android/app/src/main/jniLibs/arm64-v8a/libsylvan_row.so android/app/src/main/jniLibs/arm64-v8a/libbevy_mobile_example.so