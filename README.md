# Sylvan Row
A twin-stick hero shooter, with the ambition of avoiding hard-counter interactions, having simple but unique characters, playing on keyboard&mouse and controller, all while having a working anticheat despite being FOSS.

<img src="assets/characters/time_queen/textures/banner.png" width="300" title="Preliminary art of one of the characters" alt="An anthro lioness in armor"/>

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

## Playtestable game

- [x] Methods for server player for taking damage, healing, and secondary charge, to ease special logic.
- [x] Client also dashes accordingly
  - [x] Somewhat fixed, new networking makes it smoother.
  - [x] Fully fixed.
- [x] Finish gamemode
  - [x] Round restarts
- [x] Character picker before joining
  - [x] UI "library"
    - [x] Character picker
    - [x] Client directly connects with a desired character specified in each packet sent, since all packets are identical.
- [x] Buff system
- [x] Finish Characters
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
  - [x] The queen
    - [x] Secondary has a trail
- [x] Improve map
- [x] Simple background tile system
- [x] Sort gameobjects by height
- [x] Arbitrarily balance characters
  - [x] Raphaelle
  - [x] Cynewynn
  - [x] Hernani
- [x] Basic drawings
  - [x] Bullets
    - [x] R
      - [x] Normal
      - [x] Empowered
    - [x] C
    - [x] H
    - [x] Rotation
  - [x] Characters
    - [x] Raphaelle
    - [x] Cynewynn
    - [x] Hernani
      - [x] Wall
      - [x] Landmine
- [x] Disconnect players after inactivity
  - [x] Fix indexing issue (If a player quits, previously assigned "owner index" values become wrong.) Replace with port number?
- [x] Client network and input have separate target Hz
- [x] Server keeps sending override packets?????? lil bros a little too trigger happy

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
  - [ ] Account system
    - [ ] One way password hash
    - [ ] User database
    - [ ] Login window and allat

### After every playtest:

- [ ] Update gamemode

## Polish

- [ ] Good sound design
  - [ ] One sound for every action, etc...
- [ ] Main menu and matchmaking server
- [ ] Account system
- [ ] Better graphics
  - [ ] Animation system
  - [ ] Proper art
  - [ ] Good font
- [ ] Interpolation
  - [ ] Remove clientside dash and interpolate that too.
- [ ] Map editor
- [ ] Fix anticheat
- [ ] Canvas flipping
- [x] Smooth camera
- [ ] Find vulnerabilities
  - [ ] Big packets and false packets can crash
- [ ] Optimise size of network packets

## Publish

- [ ] Decide whether to fully host on cloud or use a VPS for tunneling
  - [ ] Find an adequate provider
- [ ] Publish on steam
- [ ] Advertise
  - [ ] Reddit?
  - [ ] Tiktok?
  - [ ] Youtube?

## Independent of playtests

- [ ] Clean up code
- [ ] Make a better looking UI
  - [x] Ability cooldowns
  - [ ] Teammates, opponents, etc.

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
