# Unnamed MOBA

A Multiplayer Online Battle Arena game, focusing on the following values:
- Being FOSS
- Being balanced
- Having brute-force anticheat
- The client not being very demanding.
- Having relatively simple but unique characters
- Having snappy movement (WASD instead of click-to-move)
- Controller compatible
- Top-down shooter style
- Hand-drawn but 3D-looking graphics

## README is incomplete ignore everything below this title. Also not accepting contributions as of now, but will gladly in the future.

run both:
```
cargo run --release --bin game server
```

## Rendering:

for rendering layers correctly, the client will be sent a pre-sorted list (by the server) of gameobjects to render in that order.

## TODO

### Immediate
- [x] Add healing, from attacks
- [x] Bullets only hit people once
- [x] Non-piercing bullets need to be deleted once they hit
- [x] Bullet hit-radius depends on character
- [x] Temporary health bar
- [x] Bullet collisions with walls
- [ ] Player collisions with walls
- [ ] Fix stupid bullet ID thing

### Irrelevant now, do after playtesting
- [ ] Add healing, passive (to be gamedesigned)
- [ ] Anticheat still doesnt work since a client can report false packet intervals. The server needs to calculate the intervals the client is sending at as an average. This will be ignored for the sake of working on the rest of the game.
- [ ] Tie together the game. (Menu, gamemodes, etc)
- [ ] Improve camera
- [ ] Animations
- [ ] Draw sprites
- [ ] Canvas flipping