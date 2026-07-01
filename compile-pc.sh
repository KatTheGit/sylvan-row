# build for Windows
mkdir target/sr_windows
rm -rf target/sr_windows/*
cargo build --bin sylvan_row --release --target x86_64-pc-windows-gnu
mv target/x86_64-pc-windows-gnu/release/sylvan_row.exe target/sr_windows/run.exe
# assets
cp -r assets target/sr_windows/assets
find target/sr_windows/assets -type f -name "*.kra" -delete
cd target
rm sr_windows.zip
zip -r sr_windows.zip sr_windows
cd ..
# build for Linux
mkdir target/sr_linux
rm -rf target/sr_linux/*
cargo build --bin sylvan_row --release --target x86_64-unknown-linux-gnu
mv target/x86_64-unknown-linux-gnu/release/sylvan_row target/sr_linux/run
# assets
cp -r assets target/sr_linux/assets
find target/sr_linux/assets -type f -name "*.kra" -delete
cd target
rm sr_linux.zip
zip -r sr_linux.zip sr_linux
cd ..

# build the server
cargo build --bin server --release --target x86_64-unknown-linux-gnu
mv target/x86_64-unknown-linux-gnu/release/server target/server-release
