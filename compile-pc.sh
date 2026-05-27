# compile the game
cargo build --bin sylvan_row --release
# remove the previous compilation
rm -rf target/compiled-game/
mkdir target/compiled-game
# copy the compiled binary into the new directory
cp target/release/sylvan_row target/compiled-game/game
# copy the assets into the new directory
cp -r assets target/compiled-game/assets
# remove all krita files. no need to distribute those.
find target/compiled-game/assets -type f -name "*.kra" -delete