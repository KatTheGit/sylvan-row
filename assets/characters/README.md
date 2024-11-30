Template Pkl character file:

```rust
health = u8

speed = f32

primary_damage = u8
primary_heal = u8
primary_cooldown = f32
primary_range = f32
primary_shot_speed = f32
primary_hit_radius = f32

secondary_damage = u8
secondary_heal = u8
secondary_hit_charge = u8
secondary_heal_charge = u8
secondary_passive_charge = u8
secondary_range = f32

dash_distance = f32
dash_cooldown = f32
dash_damage_multiplier = f32
dash_speed = f32
```

All cooldowns are in seconds. All distances are in screen percentage units (vh). `dash_damage_multiplier` is a quotient.

- `health` character's health (0-255, standadrd 100)
- `speed` character's speed (0-255, standard 100)
- `primary_damage` damage of primary attack (0-255, standard 20)
- `primary_heal` healing provided by primary attack to teammates (0-255, standard 0)
- `primary_cooldown` time between uses of attack (0.0-f32::MAX, standard 0.5)
- `primary_range` attack's reach (0.0-f32::MAX, standard 70.0)
- `primary_shot_speed` how fast the bullet travels (0.0-f32::MAX, standard 100.0)
- `primary_hit_radius` size of bullet (0.0-f32::MAX, standard 5.0)
- `secondary_damage`
- `secondary_heal`
- `secondary_hit_charge` amount secondary bar is recharged from hitting opponent (0-255, standard 50)
- `secondary_heal_charge` amount secondary bar is recharged from healing teammate (0-255, standard 50)
- `secondary_passive_charge` amount secondary bar is recharged each second (0-255, standard 0)
- `secondary_range`
- `dash_distance` distance player dashes or jumps (0.0-f32::MAX, standard 50.0)
- `dash_cooldown` minimum time between dashes (0.0-f32::MAX, standard 10.0)
- `dash_damage_multiplier` how much damage is reduced or increassed during the dash, as a quotient (0.0-f32::MAX, standard 0.0-1.0)
- `dash_speed` how fast the player moves when dashing (0.0-f32::MAX, standard 300.0)