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

# To do (not ordered)

- [ ] Implement feedback
- [ ] Health u8 -> f16
- [ ] New characters
  - [ ] Elizabeth
    - [x] Primary
    - [ ] Secondary
    - [ ] Dash
- [ ] Gamemode
- [ ] Menus
  - [ ] Pause menu, always accessible
  - [ ] Home menu and game
- [ ] Sound
  - [ ] 3D sound (get direction sound game from)
  - [ ] Volume sliders in settings screen, etc
  - [ ] Music
- [ ] Visual
  - [ ] Animation system
  - [ ] Scenery & prettier backgrounds
    - [ ] Background loader from file
  - [ ] Mirror the map
- [ ] Interpolation
  - [ ] Keep extrapolation for SIMPLE objects.
  - [ ] Remove clientside dash, make it interpolate instead
- [ ] Clean code
  - [ ] New game engine
    - [ ] Android compatible
  - [ ] Server vulnerabilities
  - [ ] Variable names, readability
  - [ ] Organisation
  - [ ] Packet size
- [ ] Anticheat
  - [ ] Packet averaging
  - [ ] Hide certain information from client

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
