# Sylvan Row

A multiplayer twin-stick hero shooter designed to be fair and fun while still allowing a high skill ceiling.

<img src="assets/characters/cynewynn/textures/banner.png" width="370" title="Preliminary art of Cynewynn" alt="An anthro lioness in armor, preliminary art for the game"/> <img src="assets/characters/temerity/textures/banner.png" width="300" title="Preliminary art of Temerity" alt="An anthro hyena in semi-casual police uniform wielding a rocket launcher, preliminary art for the game"/>

## Play

If you're interested in playtesting we'd love to have you over at our [Discord](https://discord.gg/4SbwGZeYcx)!

Otherwise there is a playable release with a working offline mode, but you'll need to host your own server if you want to try the game with others.
When you run the game, a file called `moba_ip.txt` will be created in the same directory. You can put your own server's IP(v4) address in there.

## Compile and run it yourself

You will need [the Rust compiler](https://rust-lang.org/).

To run the game client:
```sh
cargo run --bin sylvan_row --release #or ./client.sh
```

To run the server:
```sh
cargo run --bin server # or ./server.sh
```

To compile the game client (for distribution) run `compile-pc.sh` or `compile-android.sh`. To compile the server run:

```sh
cargo build --bin server --release
```

### Dependencies

On Linux, you need to additionally install the following packages to compile the code. (package names for apt)
- pkg-condfig
- libwayland-dev
- libudev-dev
- libalsa-ocaml-dev
- libx11-dev

## Other info

There is no documentation for the code yet, as a lot of it is in a temporary state.

The GDD is in `assets/README.md`.

## To do

- [ ] programming
  - [ ] Pre-game screen
    - [x] infrastructure
  - [x] Post-game screen
  - [x] gamemode
    - [x] capture point
    - [x] ui
      - [ ] all players
      - [x] game status
      - [ ] your player
  - [ ] characters
    - [ ] sága
      - [ ] design kit
    - [ ] assassin
  - [ ] bevy
    - [x] re-add friend system
    - [ ] fix android
  - [ ] ranked
  - [ ] anticheat
  - [ ] server throttling
  - [ ] chat filters
  - [ ] new parser
  - [x] standardize Z layers
  - [ ] better ui
  - [ ] status effects  
    - [x] stacks
  - [x] floating dmg numbers
  - [ ] animations
  - [ ] sfx, voicelines

## Todo 2

- [x] Fix anticheat
- [ ] Username filter
- [ ] Ranked system (?)
- [ ] Find fix for android
- [ ] Art
  - [ ] Sprites
  - [ ] Maps
  - [ ] Audio
  - [ ] UI


### Bugs

- [ ] Weird indexing errors in gameserver.
- [ ] Weird Z-layer errors regarding text.

## Reminders
- Update Wiro projectile list when new char
- Update wall list when new wall

# Notes

This was previously owned by OrnitOnGithub, my previous account, as mentioned in the [original repository](https://github.com/OrnitOnGithub/moba?tab=readme-ov-file#notice).

## Extra credits

- MylesDeGreat on deviantart for inspiration on the sword slash sprite
- posemy.art for reference images
- Inspiration
  - Assault Android Cactus
  - Battlerite
  - League of Legends