# Unnamed MOBA

A Multiplayer Online Battle Arena game, focusing on the following values:
- Being FOSS
- Being balanced
- Having brute-force anticheat
- The client not being very requiring.
- Having relatively simple but unique characters
- Having snappy movement (WASD and not click-to-move)
- Controller compatible
- Top-down shooter style
- Hand-drawn but 3D-looking graphics

## README is incomplete ignore everything below this title. Also not accepting contributions as of now, but will gladly in the future.

run both:
```
cargo run --release --bin game server
```

# Threads - client

## Thread 1: game
- handles pretty much only rendering, audio and whatnot.

## Thread 2: network sender and input listener
- Sends data to server at ludicrous frequency
- Updates player info with controller inputs, again at ludicrous frequency
- Runs like at least at 300Hz

## Thread 3: network listener
- Listens to data recieved from server
    - Updates player info, game info
    - handles overrides

# TODO:

- [ ] Split into 3 threads
- [ ] Find safe way for interthread info sharing.

# Server-Client model:

## Game basis

Player variables:
- can move
- can aim
- can shoot
- can take damage
- can heal damage after time interval
- can swap weapons and use ultimate until overheat
- can recharge ultimate

## Client's job:
Before sending packet:
- handle movement and send position to server
- send aim and movement direction to server
- send whether shooting to server, and with which weapon

After recieving server packet:
- update UI with new info
- update player positions and make them move using moveDir
- update player rotation with their aimDir

Packet (server info):
- pos: Vec2
- aimDir: Vec2
- moveDir: Vec2
- shooting: bool
- shootingSecondary: bool

## Server's job:
- recieve movement info, check if movement is legal
- instanciate bullets if player shooting
- if player not shooting with ultimate, reduce their ultimate cooldown
- if player shooting with ultimate, increase their ultimate cooldown
- calculate collisions and assign damage

Packet (sent back to client):
- Vec<Player {
    - player_position: Vec2,
    - player_movDir: Vec2,
    - player_health: usize,
    - player_ult_cooldown: usize }>

# Game design and balance rules
Goal: balance logically/mathematically, and organically later only if needed
- All characters of the same class must take the exact same amount of time to take down a full-health same-class character when continuously shooting.
- Secondaries are never direct attacks, and usually will not synergise (directly) with the primary attack.
- Healers can heal 0-100% at half the kill speed of all other characters.
- Only healers can have offensive secondaries, charged by healing teammates or passively.
- Healers' secondaries must be roughly half as strong as attacker's primaries.

# Rendering:

for rendering layers correctly, the client will be sent a pre-sorted list (by the server) of gameobjects to render in that order.