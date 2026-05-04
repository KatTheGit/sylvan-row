cargo build --bin sylvan_row --release
rm -rf target/compiled-game/
mkdir target/compiled-game
cp target/release/sylvan_row target/compiled-game/game
cp -r assets target/compiled-game/assets