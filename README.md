# Unnamed MOBA

A Multiplayer Online Battle Arena game, with the ambition of being balanced (with no hard-counter interactions), simple  but unique characters, playing like a twin stick shooter (PC and controller), while having a working anticheat despite being FOSS.

One day this game will be complete, but you can copy the repository yourself if you want a template. The `assets` directory is strictly licensed.

## Play

Tested on Windows, Linux and OSX. Not in a very playable state. There is a playtest release, but it's very rough. A properly playable release is being worked on.

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
    - [ ] Put the logic in a common function
- [x] Design some characters
- [ ] Implement first 3 characters
  - [x] Healer
    - [x] Primary (single shot)
    - [x] Dash (short dash)
    - [x] Secondary (healing aura...?)
  - [x] Sniper
    - [x] Primary (single shot)
    - [x] Dash (semi-long dash)
    - [x] Secondary (wall placement)
  - [ ] Assassin
    - [x] Primary (sword, splash)
    - [x] Dash (long dash)
    - [x] Secondary (position revert)
      - [ ] Render it
- [x] Correctly update info for each player
- [x] Extrapolation (clientside)
  - [x] Gameobjects
  - [x] Players

### Get ready for playtest 1

- [x] "Fix" getting stuck inside walls (push out)
- [x] Fix hp overheal
- [ ] Server charcter queue
  - [x] Implement
  - [ ] Test
- [x] Read IP from file
- [ ] Balance changes
- [x] Very rough art
  - [x] Character top-down views
    - [ ] Impove
  - [x] Wall sprites
  - [x] Bullets
- [x] Decent map
  - [x] Map "editor"

### Playtest 1 issues

- [x] Aiming lazer too thin, weird result on high res OLED screens (maybe?).
- [ ] Game too chaotic
  - [x] Restrict players to map bounds
  - [ ] Make characters slower
  - [ ] Visual clarity
  - [ ] Death and respawning

### Create gamemode (polish the game for PROPER playtesting)

- [x] UI
  - [x] Gamemode UI
    - [x] Make server send gamemode info
      - [x] Rounds
      - [x] Game time
      - [ ] Kills per player...? (not necessary yet)
      - [ ] Allied player's healths...?
- [ ] Create a gamemode
  - [ ] Deathmatch gamemode
  - [ ] Arena gamemode
    - [ ] Requires a death state
- [ ] Sounds
  - [ ] Find a sound engine
  - [ ] Proper sound design (not good sounds, but one sound for everything) 
    - [ ] Credit sounds when necessary in license file
- [x] Io(Kind(UnexpectedEof)) error (buffer size increased)
- [ ] Improve existing characters
  - [ ] The bunny is bland
    - [ ] Dash ability
    - [ ] Redo main attack
    - [ ] Modify secondary to heal self less
  - [ ] The queen is bland
    - [ ] Dash ability
    - [ ] Secondary has a trail (clientside)
  - [ ] The wolf just sucks
    - [ ] Dash ability
- [ ] Make the game not look like doodoo
  - [ ] Good looking tiles (TAKE INSPIRATION FROM OTHER GAMES)
    - [ ] Background tiles
      - [ ] Create a second layer of map tiles
      - [ ] Draw
    - [ ] Foreground tiles
      - [ ] Create wall types to have wall variety
      - [ ] Draw
    - [ ] Better characters?
- [ ] Clean up the code
  - [ ] More stuff in common functions
  - [ ] Different files for each thread or something
### Irrelevant now, do after playtesting

- [ ] Network packets are HUGE
- [ ] Maybe use interpolation instead of extrapolation, could result in delay, but will look far better.
- [ ] Clean up code
- [ ] Hunt for more vulnerabilities
  - [ ] If client sends big packet, server crashes
  - [ ] If client sends incorrect packet, server crashes
- [ ] Improve camera
- [ ] Vertical sorting of gameobjects and drawing layers
- [ ] Animations
- [ ] Tie together the game. (Menu, gamemodes, matchmaking server, etc)
  - [ ] Create main menu
  - [ ] Create matchmaking server
  - [ ] Allow a quick-play mode for developer use only
- [ ] Canvas flipping
- [ ] Anticheat
  - [ ] Anticheat still doesnt work since a client can report false packet intervals. The server needs to calculate the intervals the client is sending at as an average. This will be ignored for the sake of working on the rest of the game.
  - [ ] Hide certain stats if player not within visual range
- [ ] Figure out port and firewall shenanigans
  - No issues on Windows and OSX
  - [ ] Pop-up for Linux players who might need to manually make firewall rules.
  - [ ] Allow use of different ports
    - [ ] clientside
    - [ ] serverside
- [ ] Further network-related de-jittering measures than simple extrapolation.
- [ ] Map editor
- [ ] Offload work to client if possible
- [ ] Custom font?
  - [ ] Figure out TTF or make own monospace font engine

## Issues that won't be solved

- Fullcreen issue on Linux (can't exit fullscreen) (Macroquad issue)
- Icon doesn't show up on Linux (Macroquad issue)