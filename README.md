run both:
```
cargo run --bin game server
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