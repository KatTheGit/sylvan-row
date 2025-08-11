# Sylvan Row
A Multiplayer Online Battle Arena game, with the ambition of being balanced (with no hard-counter interactions), having simple  but unique characters, playing like a twin stick shooter (PC and controller), while having a working anticheat despite being FOSS.

<img src="assets/characters/time_queen/textures/banner.png" width="300" title="Preliminary art of one of the characters" alt="An anthro lioness in armor"/>

## Play

The game is not yet playable.

Run the game:
```sh
cargo run --bin game --release # or ./client.sh
```
Run the server:
```sh
cargo run --bin server --release # or ./server.sh
```

You can change the server's IP in the file that gets created in the same directory the game is run. Default is 0.0.0.0.

## Dependencies

On Linux, you need to additionally install `libudev-dev`.

# To do

## Playtestable game

- [x] Methods for server player for taking damage, healing, and secondary charge, to ease special logic.
- [ ] Client also dashes accordingly
- [x] Finish gamemode
  - [x] Round restarts
- [ ] Character picker before joining
  - [x] UI "library"
    - [ ] Character picker
    - [ ] Client directly connects with a desired character specified in each packet sent, since all packets are identical.
- [x] Buff system
- [ ] Finish Characters
  - [x] The bunny
    - [x] Primary
    - [x] Secondary
    - [x] Dash
    - [x] Passives
  - [x] The wolf
    - [x] Primary
    - [x] Secondary
    - [x] Dash
    - [x] Passives
  - [ ] The queen
    - [x] Secondary has a trail
    - [ ] Primary
    - [ ] Secondary
    - [ ] Dash
    - [ ] Passives
- [x] Improve map
- [ ] Arbitrary balance using theory
  - [ ] Bunny
  - [ ] Queen
  - [ ] Wolf
- [ ] Background tile system
- [ ] Drawings
- [ ] Maybe increase health 100->255 for higher precision, still display 0-100 clientside.

### After every playtest:

- [ ] Balance characters
- [ ] Change sizes and whatnot
- [ ] Other feedback

## Find fun gamemode + a bit of polish

- [ ] Clean up code if necessary
- [ ] Elliptic FOW instead of aspect ratio shenanigans
- [ ] Sounds
- [ ] Decent art for the characters now that they're playtest cleared.
- [ ] Custom controls
- [ ] Matchmaking server
  - [ ] Client sends request to play
  - [ ] Server put player in queue
  - [ ] Server sends player information about game server
  - [ ] Server launches Game Server
  - [ ] Client connects to Game Server
  - [ ] Profit

### After every playtest:

- [ ] Update gamemode

## Polish
- [ ] Clean up code if necessary
- [ ] Good sound design
  - [ ] One sound for every action, etc...
- [ ] Main menu and matchmaking server
- [ ] Better graphics
  - [ ] Animation system
  - [ ] Proper art
  - [ ] Good font
- [ ] Interpolation
- [ ] Map editor
- [ ] Fix anticheat
- [ ] Canvas flipping
- [ ] Smooth camera
- [ ] Find vulnerabilities
  - [ ] Big packets and false packets can crash
- [ ] Optimise size of network packets
- [ ] 

## Issues that won't be solved

- Fullcreen issue on Linux (Macroquad issue)
  - [x] Holy shit they fixed it
- Icon doesn't show up on Linux (Macroquad issue)
  - [x] Holy guacamole they fixed it

## Note

This was previously owned by OrnitOnGithub, my alt account, as mentioned [in the original repository](https://github.com/OrnitOnGithub/moba?tab=readme-ov-file#notice)