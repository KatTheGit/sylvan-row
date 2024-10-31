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

### Controls and Movement

The player has four game controls, which are independent of their picked character:
- aim
- move
- shoot
- shoot secondary

A player may move freely if unobstructed by the map, aim and shoot freely at all times, and shoot their secondary
once it is charged. These are God given rights and may not be inhibited by other players and their abilities.

### Health and Damage

When taking damage, the player's health (HP) and max health are both reduced, with max health being reduced slower. HP can be recovered after a little while of not taking damage. Max health can only be restored to its original value by healers. Healers however restore it very slow, and are better at restoring normal HP.

### Map

Maps will contain walls. Walls are a major gameplay element. They can be added (by abilities) or removed (shot down), but they cannot be easily shot or moved through.

Maps can contains holes or water. These can be shot through but cannot be moved through. No exceptions.

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

### Classes (these are uninspired guidelines, not obligations)

Healers and Controllers (assisting characters)
- Marginally better movement speed, slightly smaller hitboxes (to be evasive, as they get targeted)
- Smaller DPS

Long-rangers and damage dealers (beginner friendly)
- Only class allowed to have damage-dealing secondaries
- Lowest movement speed

Close-rangers
- Faster movement speeds (dodging and approach)
- Pack a punch
- Shorter range

### Character ideas (mindstorm)
- Bureaucrat mage
- Temporal monarch that can revert their position