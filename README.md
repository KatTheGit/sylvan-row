# Sylvan Row
A twin-stick hero shooter, with the ambition of avoiding hard-counter interactions, having simple but unique characters, playing on keyboard&mouse and controller, all while having a working anticheat despite being FOSS.

<img src="assets/characters/cynewynn/textures/banner.png" width="300" title="Preliminary art of one of the characters" alt="An anthro lioness in armor, preliminary art for the game"/>

## Play

The game's development highly relies on player feedback, and the game is currently having scheduled playtests on [discord](https://discord.gg/4SbwGZeYcx).

There is a playable release, but you'll need to host your own server if you want to try the game.

When you run the game, a file called `moba_ip.txt` will be created in the same directory. You can put your own server's IP address in there.

## Compile and run it yourself

Run the game:
```sh
cargo run --bin game --release # or ./client.sh
```
Run the server:
```sh
cargo run --bin server # or ./server.sh
```
Or build the binaries
```
cargo build --release
```

### Dependencies

On Linux, you need to additionally install `libudev-dev`, `libx11-dev` and `pkg-config` (apt package names) to compile the code.

## Other info

The GDD is in `assets/README.md`.

There is no documentation for the code yet.

Maps are made through `mapmaker.py`.

# To do

This is just for me.

## Minimal presentable game
- [ ] Fix bugs
  - [x] Elizabeth
  - [x] `thread 'main' panicked at src/bin/server.rs:1479:11` and `thread 'main' panicked at src/bin/server.rs:514:15:`
    - [x] temporary fix
  - [ ] Redo all the gamemode logic. All of it.
- [x] Implement feedback
  - [x] Nerf Raph's self heal
  - [x] Rework Elizabeth
- [ ] Characters
  - [ ] Rework Hernani secondary
  - [ ] Rework Elizabeth
    - [ ] Aim lazer shows ricochet
  - [ ] Temerity
  - [x] Wiro
    - [x] Shield (secondary)
    - [x] Primary
    - [x] Dash
    - [x] Passive
- [x] Gamemode
  - [x] 1v1 and 2v2
    - [x] Kinda
  - [x] Orb
    - [x] Spawns every 20sec, if one's not already in the game
    - [x] Can be knocked back
    - [x] Whoever gets the final blow gets some HP
  - [ ] AFTER MM-SERVER: a bit of time before the game starts
- [ ] Menus
  - [x] Pause menu, always accessible
    - [ ] Settings screen
  - [ ] Home menu and game
    - Home menu comes with matchmaking server...
- [ ] Matchmaking server
  - [ ] Client enters queue
  - [ ] If 2 players, Server launches an instance of a gameplay server
- [ ] Sound
  - [ ] Directional sound (kira crate)
  - [ ] Volume sliders in settings screen, etc
  - [ ] Music
    - [ ] beg Fancy or learn how to cook
  -  [ ] Sound effects
  -  [ ] Voicelines
     -  [ ] Character picked
     -  [ ] Character gets a kill
     -  [ ] Character gets MVP
- [ ] Visual
  - [ ] Animation system
  - [ ] Scenery & prettier backgrounds
    - [ ] Background loader from file
  - [ ] Mirror the map
- [ ] Interpolation
  - [ ] Keep extrapolation for SIMPLE objects.
  - [ ] Remove clientside dash, make it interpolate instead
- [ ] Anticheat
  - [ ] Packet averaging
  - [ ] Hide certain information from client
- [ ] Publish game
  - [ ] Steam
  - [ ] Marketing
  - [ ] Server hosting
    - AWS sucks

## Other

- [ ] Android support
  - [ ] Change game engine
- [ ] Clean up code
  - [x] IMPORTANT: Reorganise mutexes to avoid deadlocks
    - Current implementation is silly
  - [ ] Health u8 -> f16
  - [ ] Server vulnerabilities
  - [ ] Variable names, readability
  - [ ] Organisation
  - [ ] Packet size

## Reminders
- Update Wiro projectile list when new char
- Update wall list when new wall

## Issues that won't be solved

- Fullcreen issue on Linux (Macroquad issue)
  - [x] Holy shit they fixed it
- Icon doesn't show up on Linux (Macroquad issue)
  - [x] Holy guacamole they fixed it

# Notes

This was previously owned by OrnitOnGithub, my alt account, as mentioned [in the original repository](https://github.com/OrnitOnGithub/moba?tab=readme-ov-file#notice)

## Extra credits

- MylesDeGreat on deviantart for inspiration on the sword slash sprite
- posemy.art for reference images
- Inspiration
  - Assault Android Cactus
  - Battlerite