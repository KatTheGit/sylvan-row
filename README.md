# Unnamed MOBA

A Multiplayer Online Battle Arena game, focusing on the following:
- Balanced (no hard-counter interactions)
- Brute-force anticheat
- Relatively simple but unique characters
- Twin-stick top-down shooter style
- Hand-drawn but 3D-looking graphics
- Low skill floor, high skill cap (easy to learn, hard to master)

## Compile & run

```sh
cargo run --bin game --release
```
```sh
cargo run --bin server --release
```

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
- [ ] Extrapolation (clientside)
  - [x] Gameobjects
  - [ ] Players
- [ ] Clean up code x2
- [ ] Add healing, passive (to be gamedesigned)
- [ ] Design some characters
- [x] very rough UI
  - [x] Health bar and count
  - [x] Secondary attack bar and count
- [ ] Correctly update info for each player
- [ ] Implement dash mechanic

### Irrelevant now, do after playtesting

- [ ] Hunt for more vulnerabilities
  - [ ] Fix vulnerability (if client sends big packet, server crashes)
- [ ] Animations
- [ ] Improve camera
- [ ] Tie together the game. (Menu, gamemodes, etc)
- [ ] Canvas flipping
- [ ] Anticheat still doesnt work since a client can report false packet intervals. The server needs to calculate the intervals the client is sending at as an average. This will be ignored for the sake of working on the rest of the game.
- [ ] Figure out port and firewall shenanigans