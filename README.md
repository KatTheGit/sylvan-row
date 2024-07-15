run both:
```
cargo run --release --bin game server
```

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
- All characters of the same type must take the exact same amount of time to take down a full-health character when continuously shooting.
- Secondaries are never direct attacks, and usually will not synergise (directly) with the primary attack.
- Healers can heal 0-100% at half the kill speed of all other characters.
- Healers can have offensive secondaries, charged by healing teammates or passively.
- Healers' secondaries must be roughly as strong as attacker's primaries.