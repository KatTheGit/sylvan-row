# Unnamed MOBA

A Multiplayer Online Battle Arena game, focusing on the following values:
- Being balanced (no hard-counter interactions)
- Having brute-force anticheat
- Having relatively simple but unique characters
- Playing like a twin-stick top-down shooter
- Looking hand-drawn but also 3D-ish
- Being easy to learn but hard to master

## Compile & run

Tested on Windows, Linux and OSX. Not in a very playable state, so no realeases yet.

Run the game:
```sh
cargo run --bin game --release
```
Run the server:
```sh
cargo run --bin server --release
```

You can change the server's IP in `src/bin/game.rs:~211`

## Dependencies

On Linux, you need to additionally install `libudev-dev`.

# TODO

### Immediate
- [x] Add healing, from attacks
- [x] Bullets only hit people once
- [x] Non-piercing bullets need to be deleted once they hit
- [x] Bullet hit-radius depends on character
- [x] Temporary health bar
- [x] Bullet collisions with walls
- [x] Fix (delete) bullet ID thingy
- [x] Player collisions with walls
- [x] Might need to increase the size of the network packet reception buffers.
- [x] very rough UI
  - [x] Health bar and count
  - [x] Secondary attack bar and count
- [x] Implement dash mechanic
  - [x] Client sends dash info
  - [x] Server cooks
  - [ ] Client cooperates
- [x] Design some characters
- [ ] Implement first 3 characters
  - [ ] Healer
    - [x] Primary (single shot)
    - [x] Dash (short dash)
    - [ ] Secondary (healing aura...?)
  - [ ] Sniper
    - [x] Primary (single shot)
    - [x] Dash (semi-long dash)
    - [ ] Secondary (wall placement)
  - [ ] Assassin
    - [x] Primary (sword, splash)
    - [x] Dash (long dash)
    - [ ] Secondary (position revert)
- [x] Correctly update info for each player
- [x] Extrapolation (clientside)
  - [x] Gameobjects
  - [x] Players
- [ ] Fix getting stuck inside walls (push out)

### Irrelevant now, do after playtesting

- [ ] Clean up code x2
- [ ] Hunt for more vulnerabilities
  - [ ] Fix vulnerability (if client sends big packet, server crashes)
- [ ] Animations
- [ ] Improve camera
- [ ] Tie together the game. (Menu, gamemodes, etc)
- [ ] Canvas flipping
- [ ] Anticheat still doesnt work since a client can report false packet intervals. The server needs to calculate the intervals the client is sending at as an average. This will be ignored for the sake of working on the rest of the game.
- [ ] Figure out port and firewall shenanigans
  - No issues on Windows and OSX
  - [ ] Pop-up for Linux players who might need to manually make firewall rules.
  - [ ] Allow use of different ports
    - [ ] clientside
    - [ ] serverside
- [ ] Further network-related de-jittering measures than simple extrapolation.
- [ ] Map editor
- [ ] Offload work to client if possible

## Issues that won't be solved

- Fullcreen issue on Linux (can't exit fullscreen) (Macroquad issue)