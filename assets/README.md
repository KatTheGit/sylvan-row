# Game design document

This document specifies all design rules and choices for this videogame.

## Basic game description

A multiplayer arena fighting game where players fight with their pick amongst a cast of varied characters. It focuses on the following values:
- Having balanced characters
- Having relatively simple but unique characters
- Having snappy movement (WASD instead of click-to-move)
- Being ontroller compatible
- Being a top-down shooter
- Having hand-drawn but 3D-looking graphics

## Base mechanics

There are, independently of the picked character, three types of abilities. These differ by character.
- Primary attack
- Secondary attack
- Dash
And also character dependent statistics
- Max health
- Movement speed

### Controls and Movement

The player has five game controls, which are independent of their picked character:
- aim
- move
- shoot
- shoot secondary
- Dash

A player may move freely if unobstructed by the map, aim and shoot freely at all times, and shoot their secondary
once it is charged. These are God given rights and may not be inhibited by other players and their abilities.

### Map

Maps will contain walls. Walls are a major gameplay element. They can be added (by abilities) or removed (shot down), but they cannot be easily shot or moved through.

Maps can contains holes or water. These can be shot through but cannot be moved through. No exceptions.

## Classes

These are three classes that create a ternary system. It creates a rock-paper-scissors style countering triangle. Of course all "counters" are soft-counters. There should be no hardcounter matchups.

### Healer

The second fastest class. Are very evasive and can easily avoid big bullets but not enough to evade assassins. They "counter" rangers' defensive playstyles by simply healing when the ranger plays too defensively. They have the second longest range.

Healers excel at supporting assassins in battle.

### Assassin

The fastest class. They specialise in high "burst" mobility and have a higher damage output. They have the shortest range. They easily catch up to healers, but may have a bit more trouble with rangers.

Assassins excel at supporting rangers by protecting them from threats like other assassins or healers.

### Ranger

The slowest but most defensive class. They have a very long range and defensive abilities that make them harder to approach. Assassins have more trouble with them than healers, if the ranger plays carefully.

They excel at helping healers in battle, as they can help defend  them from assassins.

## Random shit I wrote and can't be assed to delete

### Health and Damage

When taking damage, the player's health (HP) and max health are both reduced, with max health being reduced slower. HP can be recovered after a little while of not taking damage. Max health can only be restored to its original value by healers. Healers however restore it very slow, and are better at restoring normal HP.

## Characters

### Character balance rules
Goal: balance logically/mathematically, and organically later only if needed
- All characters of the same class (or that are similar) must take the exact same amount of time to take down a full-health same-class character when continuously shooting. (Fire rate is inversely proportional to damage dealt.)
- Secondaries are never direct attacks, and usually will not combo (directly) with the primary attack.
- Healers can heal 0-100% at half the kill speed of all other characters.
- Only healers can have offensive secondaries, charged by healing teammates or passively.
Of course all of this will be kept or trashed depending on playtester opinions.
- Attack Range is inversely proportional to movement speed.
- Bullet size is inversely proportional to bullet speed.
- The difference between class of any stat cannot be too big. (<20%)