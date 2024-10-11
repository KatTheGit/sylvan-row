# Unnamed MOBA

A Multiplayer Online Battle Arena game, focusing on the following:
- Balanced (no hard-counter interactions)
- Brute-force anticheat
- Relatively simple but unique characters
- Twin-stick shooter style / snappy WASD movement
- Hand-drawn but 3D-looking graphics
- Low skill floor, high skill cap (easy to learn, hard to master)
- Being FOSS

## README is incomplete ignore everything below this title. Also not accepting contributions as of now, but will gladly in the future.

## TODO

### Immediate
- [x] Add healing, from attacks
- [x] Bullets only hit people once
- [x] Non-piercing bullets need to be deleted once they hit
- [x] Bullet hit-radius depends on character
- [x] Temporary health bar
- [x] Bullet collisions with walls
- [x] Fix bullet ID thingy
- [ ] Player collisions with walls (done clientside)
- [ ] Might need to increase the size of the network packet reception buffers.
- [ ] Clean up code x2

### Irrelevant now, do after playtesting
- [ ] Add healing, passive (to be gamedesigned)
- [ ] Tie together the game. (Menu, gamemodes, etc)
- [ ] Improve camera
- [ ] Animations
- [ ] Draw sprites
- [ ] Canvas flipping
- [ ] Anticheat still doesnt work since a client can report false packet intervals. The server needs to calculate the intervals the client is sending at as an average. This will be ignored for the sake of working on the rest of the game.
- [ ] Figure out ports and firewall shenanigans