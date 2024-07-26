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

## Game design and balance rules
Goal: balance logically/mathematically, and organically later only if needed
- All characters of the same class must take the exact same amount of time to take down a full-health same-class character when continuously shooting.
- Secondaries are never direct attacks, and usually will not synergise (directly) with the primary attack.
- Healers can heal 0-100% at half the kill speed of all other characters.
- Only healers can have offensive secondaries, charged by healing teammates or passively.
- Healers' secondaries must be roughly half as strong as attacker's primaries.

## Rendering:

for rendering layers correctly, the client will be sent a pre-sorted list (by the server) of gameobjects to render in that order.