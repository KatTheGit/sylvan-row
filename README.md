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

- [ ] Characters
  - [ ] Rework Hernani secondary
  - [ ] Josey
    - [ ] Aim lazer shows ricochet
- [ ] Innovate gamemode
- [x] Rough menu
  - [x] Pause menu, always accessible
    - [ ] Settings screen
      - [ ] Text input field
      - [ ] Input bar
      - [ ] Checkbox
      - [ ] Drop-down menu
      - [ ] Keybind input field
  - [x] Home menu
    - [x] Tabs
      - [x] Play
      - [x] Heroes
      - [x] Tutorial
- [ ] Matchmaking server ![mindmap](mm-mindmap.png)
  - [ ] **Part 1** (for open beta)
    - [ ] Player data
      - [ ] Username
      - [ ] Password (hashed)
      - [ ] Friends
      - [ ] Muted players is CLIENTSIDE
      - [ ] Ban status
    - [ ] Basic authentication system
      - [ ] Create account
        - [ ] Profanity filter
      - [ ] Log in
    - [ ] Chat
      - [ ] Channels
      - [ ] Friends
    - [ ] Game request
      - [ ] Lobby
      - [ ] Character
    - [ ] Pre-game info
    - [ ] Post-game info
    - [ ] Fleet managment
      - [ ] Error handling
    - [ ] Logs
      - [ ] Chat logs (for moderation)
      - [ ] Server crash logs
  - [ ] **Part 2** (for release)
    - [ ] Steam integration
      - [ ] Accounts
      - [ ] In-game purchases
        - [ ] Inform game request
      - [ ] Display name & change display name
- [ ] Sound
  - [ ] Directional sound (kira crate)
  - [ ] Volume sliders in settings screen, etc
  - [ ] Music
    - [ ] beg Fancy or learn how to cook
  -  [ ] Sound effects
  -  [ ] Voicelines
    -  [ ] Character picked
    -  [ ] Character gets a kill
    -  [ ] Character wins
- [ ] Visual
  - [ ] Animation system
  - [ ] Scenery & prettier backgrounds
    - [ ] Background loader from file
  - [ ] Mirror the map
  - [ ] Revamp menu
- [ ] Interpolation
  - [ ] Self-interpolation on dashes and wallrides
  - [ ] Player interpolation for other players
- [ ] Anticheat
  - [ ] Packet averaging
  - [ ] Hide certain information from client
- [ ] Publish game
  - [ ] Steam
  - [ ] Marketing
  - [ ] Server hosting
    - AWS sucks
- [ ] Android port
  - [ ] Android-specific controls (devicequery doesnt work)

## Other


- [ ] Change game engine
- [ ] Clean up code
  - [x] IMPORTANT: Reorganise mutexes to avoid deadlocks
    - Current implementation is silly
  - [ ] Health u8 -> f16
  - [ ] Server vulnerabilities
  - [ ] Variable names, readability
  - [ ] Organisation
    - [ ] structure into proper modules, like networking, maths, players, ui, etc...
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