# Sylvan Row
A twin-stick hero shooter, with the ambition of avoiding hard-counter interactions, having simple but unique characters, playing on keyboard&mouse and controller, all while having a working anticheat despite being FOSS.

<img src="assets/characters/cynewynn/textures/banner.png" width="300" title="Preliminary art of one of the characters" alt="An anthro lioness in armor, preliminary art for the game"/>

## Play

The game is currently having scheduled playtests at https://discord.gg/4SbwGZeYcx. There is a playable release, and its default IP is set to a random AWS server.

Run the game:
```sh
cargo run --bin game --release # or ./client.sh
```
Run the server:
```sh
cargo run --bin server --release # or ./server.sh
```
Or yk just build the binaries
```
cargo build --release
```

When you run the game, a file called `moba_ip.txt` will be created in the same directory. It contains the default IP address to the current game server. You can change it to your own server, or change the default address in `common.rs`.

## Dependencies

On Linux, you need to additionally install `libudev-dev`, `libx11-dev` and `pkg-config` (apt package names).

# To do

## Minimal presentable game

- [x] Implement feedback
  - [x] Nerf Raph's self heal
  - [x] Rework Elizabeth
- [ ] New characters (6 total at least)
  - [x] Elizabeth (rename tho)
  - [ ] An assassin
  - [ ] Wiro (rename tho)
- [x] Gamemode
  - [ ] 1v1 and 2v2
    - [x] Kinda
  - [ ] Orb
    - [ ] Spawns every 30sec, if one's not already in the game
    - [ ] Can be knocked back
    - [ ] Whoever gets the final blow gets some HP
- [ ] Menus
  - [x] Pause menu, always accessible
    - [ ] Settings screen
  - [ ] Home menu and game
    - Home menu comes with matchmaking server...
- [ ] Matchmaking server
  - [ ] Client enters queue
  - [ ] If 2 players, Server launches an instance of a gameplay server
- [ ] Sound
  - [ ] Directional sound (akira crate)
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

## Other

- [ ] Android support
  - [ ] Change game engine
- [ ] Clean up code
  - [ ] IMPORTANT: Reorganise mutexes to avoid deadlocks
    - Current implementation is silly
  - [ ] Health u8 -> f16
  - [ ] Server vulnerabilities
  - [ ] Variable names, readability
  - [ ] Organisation
  - [ ] Packet size

## Issues that won't be solved

- Fullcreen issue on Linux (Macroquad issue)
  - [x] Holy shit they fixed it
- Icon doesn't show up on Linux (Macroquad issue)
  - [x] Holy guacamole they fixed it

## Note

This was previously owned by OrnitOnGithub, my alt account, as mentioned [in the original repository](https://github.com/OrnitOnGithub/moba?tab=readme-ov-file#notice)

## Extra credits

- MylesDeGreat on deviantart for inspiration on the sword slash sprite
- posemy.art for reference images
